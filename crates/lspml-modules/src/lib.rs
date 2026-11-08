use std::{
    collections::HashMap,
    fs,
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ModuleMappings(HashMap<String, Module>);

#[derive(Clone, Debug, Deserialize, Hash)]
pub struct Module {
    pub path: String,
}

impl Deref for ModuleMappings {
    type Target = HashMap<String, Module>;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

pub(crate) static MODULE_MAPPINGS: OnceLock<Arc<Mutex<ModuleMappings>>> = OnceLock::new();

pub fn init_module_mappings_from_file(file: &str) -> Result<()> {
    MODULE_MAPPINGS
        .set(Arc::new(Mutex::new(ModuleMappings(
            fs::read_to_string(&file)
                .map_err(|err| anyhow::anyhow!("failed to read {}: {}", file, err))
                .and_then(|text| {
                    serde_json::from_str(&text)
                        .map_err(|err| anyhow::anyhow!("could not parse json in {}: {}", file, err))
                })?,
        ))))
        .map_err(|_| anyhow::anyhow!("could not initialize module mappings; mutex poisoned"))?;
    log::info!(
        "created module mappings from {}: {:?}",
        file,
        MODULE_MAPPINGS.get().unwrap()
    );
    return Ok(());
}

pub fn init_empty_module_mappings() -> Result<()> {
    MODULE_MAPPINGS
        .set(Arc::new(Mutex::new(ModuleMappings(HashMap::new()))))
        .map_err(|_| anyhow::anyhow!("could not initialize module mappings; mutex poisoned"))?;
    log::info!(
        "created empty module mappings: {:?}",
        MODULE_MAPPINGS.get().unwrap()
    );
    return Ok(());
}

pub fn all_modules<'a>() -> Vec<(String, Module)> {
    return MODULE_MAPPINGS
        .get()
        .expect("module mappings not initialized")
        .lock()
        .expect("module mappings mutex poisoned")
        .iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
}

pub fn find_module_by_name(module: &str) -> Option<Module> {
    return MODULE_MAPPINGS
        .get()
        .expect("module mappings not initialized")
        .lock()
        .expect("module mappings mutex poisoned")
        .get(module)
        .cloned();
}

pub fn find_module_for_file(file: &Path) -> Option<Module> {
    return MODULE_MAPPINGS
        .get()
        .expect("module mappings not initialized")
        .lock()
        .expect("module mappings mutex poisoned")
        .iter()
        .find_map(|(_, module)| file.strip_prefix(&module.path).ok().map(|_| module.clone()));
}
