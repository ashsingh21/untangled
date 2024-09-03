use std::{fs::File, path::Path};

use tracing::info;

pub enum FileExtension {
    PDF,
    DOCX,
    XLXS,
    XLS,
    UNKNOWN,
    RS,
    TOML,
    JSON,
    MD,
    TXT
}

impl FileExtension {

    pub fn is_supported(&self) -> bool {
        match self {
            FileExtension::UNKNOWN => false,
            _ => true,
        }
    }

    pub fn from_filepath(filepath: &Path) -> Self {
        let extension = filepath.extension();
        if extension.is_none() {
            return FileExtension::UNKNOWN;
        }
        let extension = extension.unwrap().to_str().unwrap();
        match extension {
            "pdf" => FileExtension::PDF,
            "docx" => FileExtension::DOCX,
            "xlsx" => FileExtension::XLXS,
            "xls" => FileExtension::XLS,
            "txt" => FileExtension::TXT,
            _ => {
                info!("unknown file extension: {:?}", extension);
                FileExtension::UNKNOWN
            }
        }
    }
}