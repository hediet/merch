use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ParsedMerchDoc<'t> {
    pub existing_files: Vec<ExistingFileInfo>,
    pub updated_files: HashMap<usize, UpdatedFileInfo<'t>>,
    pub base_path: PathBuf,
}

#[derive(Debug)]
pub struct ExistingFileInfo {
    pub idx: usize,
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Debug)]
pub struct UpdatedFileInfo<'t> {
    pub idx: usize,
    pub new_path: PathBuf,
    pub content: Option<&'t str>,
}
