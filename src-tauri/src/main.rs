// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use indexer::DocIndexer;
use serde::Serialize;
use tantivy::{Document, TantivyDocument};
use tracing::info;
mod indexer;
mod file_extension;

fn main() {
    tracing_subscriber::fmt::init();
    let doc_indexer = indexer::DocIndexer::try_new().expect("Failed to create DocIndexer");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(doc_indexer)
        .invoke_handler(tauri::generate_handler![search, index, openfile])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug, Serialize)]
struct SearchResult {
    score: f32,
    doc: String,
}

type SearchResults = Vec<SearchResult>;

#[tauri::command]
fn search(indexer: tauri::State<DocIndexer>, query: &str) -> SearchResults {
    info!("Searching for: {:?}", query);
    let index_reader = indexer.index_reader.clone();
    let schema = indexer.index.schema();
    let searcher = index_reader.searcher();
    let query = indexer
        .query_parser
        .parse_query(query)
        .expect("Failed to parse query");

    let top_docs = searcher
        .search(&query, &tantivy::collector::TopDocs::with_limit(10))
        .expect("Failed to search");

    let mut search_results = SearchResults::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument =
            searcher.doc(doc_address).expect("Failed to retrieve doc");
        let parsed = retrieved_doc.to_json(&schema);
        info!("Found doc: {:?}", parsed);
        search_results.push(SearchResult { score, doc: parsed });
    }
    search_results
}

#[tauri::command]
fn index(indexer: tauri::State<DocIndexer>, user_supplied_paths: Vec<PathBuf>) {
    let directories_being_watched = indexer.directories_being_watched();
    // check what paths are already being watched and add the new ones and delete the ones that are not in paths
    for path in &directories_being_watched {
        if !user_supplied_paths.contains(&path.into()) {
            // TODO: how to remove data from tantivy index?
            indexer.remove_directory(path.clone().into()).expect("Failed to remove directory");
            info!("Removed directory: {:?}", path);
        }
    }

    for path in user_supplied_paths {
        if !directories_being_watched.contains(&path.to_string_lossy().to_string()) {
            let res = indexer.index_directory(path.clone());
            match res {
                Ok(_) => info!("Indexed directory: {:?}", path),
                Err(e) => info!("Failed to index directory: {:?}, error: {:?}", path, e),
            }
        }
    }
}

#[tauri::command]
fn openfile(path: PathBuf) -> tauri::Result<()> {
    let path = path.to_string_lossy().to_string();
    info!("Opening file in browser: {:?}", path);
    open::with(path, "open")?;

    Ok(())
}
