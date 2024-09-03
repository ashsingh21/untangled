use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use calamine::{Reader, Xlsx};
use docx_rs::read_docx;
use notify::{Event, EventKind, Result};

use notify::{RecommendedWatcher, Watcher};
use serde_json::Value;
use tantivy::{Index, IndexWriter, ReloadPolicy, TantivyDocument};
use tracing::{debug, info};

use tempfile::TempDir;

use crate::file_extension::FileExtension;

fn read_docx_children(node: &Value, content: &mut String) {
    if let Some(children) = node["data"]["children"].as_array() {
        children.iter().for_each(|child| {
            if child["type"] != "text" {
                read_docx_children(child, content);
            } else {
                let text = child["data"]["text"]
                    .as_str()
                    .expect("Failed to get text from docx");
                let text = text.replace("\n", " ");
                let text = format!("{} ", text);
                content.push_str(&text);
            }
        });
    }
}

pub fn file_to_string(file: &Path) -> anyhow::Result<String> {
    let extension = FileExtension::from_filepath(file);
    match extension {
        FileExtension::PDF => {
            let text = pdf_extract::extract_text(file);
            if text.is_err() {
                return Err(anyhow::anyhow!("Failed to extract text from PDF"));
            }
            Ok(text.unwrap())
        }
        FileExtension::DOCX => {
            info!("parsing as docx file: {:?}", file);
            let mut buf = Vec::new();
            std::fs::File::open(file)?.read_to_end(&mut buf)?;
            let data: Value = serde_json::from_str(&read_docx(&buf)?.json())?;
            let mut content = String::new();
            if let Some(children) = data["document"]["children"].as_array() {
                children.iter().for_each(|node| {
                    read_docx_children(node, &mut content);
                });
            }
            Ok(content)
        }
        FileExtension::XLXS => {
            info!("parsing as xlxs or xls file: {:?}", file);
            let mut workbook: Xlsx<_> = calamine::open_workbook(file)?;
            let mut content = String::new();
            let sheets = workbook.sheet_names().to_owned();
            for sheet_name in sheets {
                let sheet = workbook
                    .worksheet_range(&sheet_name)
                    .expect("Sheet not found")?;
                for row in sheet.rows() {
                    for cell in row {
                        let cell = format!("{} ", cell);
                        content.push_str(&cell);
                    }
                }
            }
            Ok(content)
        }
        FileExtension::UNKNOWN | _ => {
            debug!("Unknown file extension: {:?}", file);
            Ok("".to_string())
        }
    }
}

struct IndexMsg {
    filename: PathBuf,
    content: String,
}

pub struct DocIndexer {
    pub index: Index,
    pub query_parser: tantivy::query::QueryParser,
    pub index_reader: tantivy::IndexReader,
    tx: std::sync::mpsc::Sender<IndexMsg>,
    watchers: Arc<Mutex<RefCell<HashMap<String, DirectoryWatcher>>>>,
    _temp_dir: Arc<TempDir>,
}

impl DocIndexer {
    pub fn try_new() -> anyhow::Result<Self> {
        let index_path = Arc::new(TempDir::new()?);

        let mut schema_builder = tantivy::schema::SchemaBuilder::new();
        schema_builder.add_text_field("filename", tantivy::schema::TEXT | tantivy::schema::STORED);
        schema_builder.add_text_field("content", tantivy::schema::TEXT);

        let schema = schema_builder.build();
        let filename = schema
            .get_field("filename")
            .expect("filename field not found");
        let content = schema
            .get_field("content")
            .expect("content field not found");

        let index = tantivy::Index::create_in_dir(index_path.as_ref(), schema.clone())?;
        
        let query_parser = tantivy::query::QueryParser::for_index(&index, vec![filename, content]);
        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;
        let mut index_writer: IndexWriter = index.writer(50_000_000)?;

        let (tx, rx) = std::sync::mpsc::channel::<IndexMsg>();

        thread::spawn(move || {
            let filename_field = schema
                .get_field("filename")
                .expect("filename field not found");
            let content_field = schema
                .get_field("content")
                .expect("content field not found");
            info!("Indexing thread started...");
            loop {
                let msg = rx.recv().expect("Failed to receive message");
                info!("Indexing file: {:?}", msg.filename);
                let mut doc = TantivyDocument::default();
                doc.add_text(filename_field, msg.filename.to_string_lossy());
                doc.add_text(content_field, msg.content);

                index_writer
                    .add_document(doc)
                    .expect("Failed to add document");
                index_writer.commit().expect("Failed to commit");
            }
        });

        Ok(Self {
            index,
            watchers: Arc::new(Mutex::new(RefCell::new(HashMap::new()))),
            tx,
            query_parser,
            index_reader,
            _temp_dir: index_path,
        })
    }

    pub fn remove_directory(&self, path: PathBuf) -> anyhow::Result<()> {
        let watchers = self.watchers.lock().expect("Failed to lock watchers");
        let mut watchers = watchers.borrow_mut();
        watchers.remove(&path.to_string_lossy().to_string());
        Ok(())
    }

    pub fn directories_being_watched(&self) -> Vec<String> {
        self.watchers.lock().expect("Failed to lock watchers")
            .borrow()
            .keys()
            .map(|k| k.to_string())
            .collect()
    }

    pub fn index_directory(&self, path: PathBuf) -> anyhow::Result<()> {
        if self
            .watchers
            .lock()
            .expect("Failed to lock watchers")
            .borrow()
            .contains_key(&path.to_string_lossy().to_string())
        {
            info!("Already indexed directory: {:?}", path);
            return Ok(());
        }
        // TODO: save first time state somewhere?
        let first_time = true;
        if first_time {
            let tx = self.tx.clone();
            let path = path.clone();
            thread::spawn( move || {
                info!("First time indexing...");
                // walk the directory and add all files to the index
                let walker = walkdir::WalkDir::new(&path);
                for entry in walker {
                    let entry = entry.expect("Failed to get entry");
                    if entry.file_type().is_file() {
                        let extension = FileExtension::from_filepath(&entry.path());
    
                        if !extension.is_supported() {
                            info!("Unsupported file format: {:?}", entry.path());
                            continue;
                        }
    
    
                        let contents = file_to_string(&entry.path());
                        if contents.is_err() {
                            info!("Failed to extract text from file: {:?}", entry.path());
                            continue;
                        }
                        let contents = contents.unwrap();
                        let msg = IndexMsg {
                            filename: entry.path().to_path_buf(),
                            content: contents,
                        };
                        tx.send(msg).expect("Failed to send message");
                    }
                }
            });


        }
        let tx = self.tx.clone();
        let mut watcher =
            DirectoryWatcher::new(path.clone(), move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Modify(modify_kind) => match modify_kind {
                            notify::event::ModifyKind::Data(_) => {
                                info!("indexing file: {:?}", event.paths);
                                let file = event.paths.first().expect("No file path found");
                                // TODO: use buffered reader
                                let contents = file_to_string(file);
                                if contents.is_err() {
                                    info!("Failed to extract text from file: {:?}", file);
                                    return;
                                }

                                let contents = contents.unwrap();
                                let msg = IndexMsg {
                                    filename: file.to_path_buf(),
                                    content: contents,
                                };
                                tx.send(msg).expect("Failed to send message");
                            }
                            notify::event::ModifyKind::Name(_) => {
                                debug!("Name: {:?}", event.paths);
                            }
                            _ => {
                                debug!("Other: {:?}", event);
                            }
                        },
                        EventKind::Create(_) => {
                            debug!("Create: {:?}", event);
                        }
                        EventKind::Remove(_) => {
                            debug!("Remove: {:?}", event);
                        }
                        EventKind::Any => {
                            debug!("Any: {:?}", event);
                        }
                        _ => {
                            debug!("Other: {:?}", event);
                        }
                    }
                }
            })?;
        watcher.watch()?;
        self.watchers
            .lock()
            .expect("lock poisoned")
            .borrow_mut()
            .insert(path.to_string_lossy().to_string(), watcher);
        Ok(())
    }
}

struct DirectoryWatcher {
    path: PathBuf,
    watcher: RecommendedWatcher,
}

impl DirectoryWatcher {
    fn new(path: PathBuf, event_handler: impl notify::EventHandler) -> anyhow::Result<Self> {
        let watcher = notify::recommended_watcher(event_handler)?;

        Ok(Self { path, watcher })
    }

    fn watch(&mut self) -> Result<()> {
        println!("Watching: {:?}", self.path);
        self.watcher
            .watch(&self.path, notify::RecursiveMode::Recursive)
    }
}
