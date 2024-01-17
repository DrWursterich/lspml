use lsp_types::Url;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

#[derive(Clone, Debug)]
pub(crate) struct Document {
    pub(crate) text: String,
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
