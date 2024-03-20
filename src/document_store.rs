use anyhow::{Error, Result};
use lsp_types::Url;
use std::{
    collections::HashMap,
    fs,
    str::FromStr,
    sync::{Arc, Mutex, OnceLock},
};
use tree_sitter::{Parser, Tree};

#[derive(Clone, Debug)]
pub(crate) struct Document {
    pub(crate) text: String,
    pub(crate) tree: Tree,
}

impl Document {
    pub(crate) fn new(uri: &Url) -> Result<Document> {
        return match uri.to_file_path() {
            Ok(path) if path.exists() => fs::read_to_string(path.to_owned()).map_err(Error::from),
            Ok(path) => Result::Err(anyhow::anyhow!("file {:?} does not exist", path)),
            Err(_) => Result::Err(anyhow::anyhow!("failed to read file path from uri {}", uri)),
        }
        .and_then(|text| Document::from_str(&text));
    }
}

impl FromStr for Document {
    type Err = Error;

    fn from_str(text: &str) -> Result<Document> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_spml::language())?;
        return parser
            .parse(&text, None)
            .map(|tree| Document {
                text: text.to_string(),
                tree,
            })
            .ok_or_else(|| anyhow::anyhow!("failed to parse text: {}", text));
    }
}

fn document_store() -> &'static Arc<Mutex<HashMap<Url, Document>>> {
    static DOCUMENT_STORE: OnceLock<Arc<Mutex<HashMap<Url, Document>>>> = OnceLock::new();
    return DOCUMENT_STORE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
}

pub(crate) fn get(uri: &Url) -> Option<Document> {
    return document_store()
        .lock()
        .expect("document_store mutex poisoned")
        .get(&uri)
        .cloned();
}

pub(crate) fn put(uri: &Url, document: Document) -> Document {
    document_store()
        .lock()
        .expect("document_store mutex poisoned")
        .insert(uri.clone(), document.clone());
    return document;
}
