use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex, OnceLock},
};

use anyhow::Result;
use lsp_types::Uri;
use tree_sitter::Parser;

use lspml_spml::Tree;

#[derive(Clone, Debug)]
pub struct Document {
    pub text: Box<str>,
    pub tree: Tree,
}

impl Document {
    pub fn new(text: String) -> Result<Document> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_spml::language())?;
        match parser.parse(&text, None) {
            Some(tree) => Tree::new(tree, &text).map(|tree| Document {
                text: text.into(),
                tree,
            }),
            None => Err(anyhow::anyhow!("failed to parse text: {}", text)),
        }
    }

    pub fn from_uri(uri: &Uri) -> Result<Document> {
        let path = uri_to_path(uri)?;
        fs::read_to_string(&path)
            .map_err(|err| anyhow::anyhow!("cannot read file {:?}: {}", path, err))
            .and_then(Document::new)
    }
}

fn document_store() -> &'static Arc<Mutex<HashMap<Uri, Document>>> {
    static DOCUMENT_STORE: OnceLock<Arc<Mutex<HashMap<Uri, Document>>>> = OnceLock::new();
    DOCUMENT_STORE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

pub fn get(uri: &Uri) -> Option<Document> {
    return document_store()
        .lock()
        .expect("document_store mutex poisoned")
        .get(uri)
        .cloned();
}

pub fn put(uri: &Uri, document: Document) -> Document {
    document_store()
        .lock()
        .expect("document_store mutex poisoned")
        .insert(uri.clone(), document.clone());
    document
}

pub fn uri_to_path(uri: &Uri) -> Result<PathBuf> {
    url::Url::parse(uri.as_str())?
        .to_file_path()
        .map_err(|_| anyhow::anyhow!("not a file uri: {}", uri.as_str()))
}

pub fn path_to_uri(path: &Path) -> Result<Uri> {
    let url = url::Url::from_file_path(path)
        .map_err(|_| anyhow::anyhow!("not an absolute path: {:?}", path))?;
    Ok(Uri::from_str(url.as_str())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_uri_round_trip_with_special_characters() {
        let path = Path::new("/tmp/a b/c ä.spml");
        let uri = path_to_uri(path).unwrap();
        assert_eq!(uri.as_str(), "file:///tmp/a%20b/c%20%C3%A4.spml");
        assert_eq!(uri_to_path(&uri).unwrap(), path);
    }
}
