use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

use anyhow::{Error, Result};
use lsp_types::Uri;
use tree_sitter::Parser;

use parser::Tree;

#[derive(Clone, Debug)]
pub struct Document {
    pub text: Box<str>,
    pub tree: Tree,
}

impl Document {
    pub fn new(text: String) -> Result<Document> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_spml::language())?;
        return match parser.parse(&text, None) {
            Some(tree) => Tree::new(tree, &text).map(|tree| Document {
                text: text.into(),
                tree,
            }),
            None => Err(anyhow::anyhow!("failed to parse text: {}", text)),
        };
    }

    pub fn from_uri(uri: &Uri) -> Result<Document> {
        return match Path::new(uri.path().as_str()) {
            path if path.exists() => fs::read_to_string(path.to_owned())
                .map(Document::new)
                .map_err(Error::from),
            path => Err(anyhow::anyhow!("file {:?} does not exist", path)),
        }?;
    }
}

fn document_store() -> &'static Arc<Mutex<HashMap<Uri, Document>>> {
    static DOCUMENT_STORE: OnceLock<Arc<Mutex<HashMap<Uri, Document>>>> = OnceLock::new();
    return DOCUMENT_STORE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
}

pub fn get(uri: &Uri) -> Option<Document> {
    return document_store()
        .lock()
        .expect("document_store mutex poisoned")
        .get(&uri)
        .cloned();
}

pub fn put(uri: &Uri, document: Document) -> Document {
    document_store()
        .lock()
        .expect("document_store mutex poisoned")
        .insert(uri.clone(), document.clone());
    return document;
}
