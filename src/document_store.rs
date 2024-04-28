use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex, OnceLock},
};

use anyhow::{Error, Result};
use lsp_types::Url;
use tree_sitter::Parser;

use crate::parser::{Header, Tree};

#[derive(Clone, Debug)]
pub(crate) struct Document {
    pub(crate) text: String,
    pub(crate) tree: Tree,
}

impl Document {
    pub(crate) fn new(text: String) -> Result<Document> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_spml::language())?;
        return match parser.parse(&text, None) {
            Some(tree) => {
                match Tree::new(tree, &text) {
                    Ok(tree) => Ok(Document { text, tree }),
                    Err(err) => {
                        log::info!("could not parse syntax tree: {}", err);
                        // TODO: should be able to handle incomplete trees!
                        Ok(Document {
                            text,
                            tree: Tree {
                                header: Header {
                                    java_headers: vec![],
                                    taglib_imports: vec![],
                                },
                                tags: vec![],
                            },
                        })
                    }
                }
            }
            None => return Result::Err(anyhow::anyhow!("failed to parse text: {}", text)),
        };
    }

    pub(crate) fn from_uri(uri: &Url) -> Result<Document> {
        return match uri.to_file_path() {
            Ok(path) if path.exists() => fs::read_to_string(path.to_owned())
                .map(|text| Document::new(text))
                .map_err(Error::from),
            Ok(path) => Err(anyhow::anyhow!("file {:?} does not exist", path)),
            Err(_) => Err(anyhow::anyhow!("failed to read file path from uri {}", uri)),
        }?;
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
