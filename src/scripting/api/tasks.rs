use once_cell::sync::OnceCell;
use rune::{ContextError, Hash, Module};
use rune::runtime::{SyncFunction, Value};
use std::collections::HashMap;
use std::sync::Mutex;

static REGISTRY: OnceCell<Mutex<HashMap<String, SyncFunction>>> = OnceCell::new();

pub fn register(name: &str, f: SyncFunction) {
    REGISTRY.get_or_init(|| {
        Mutex::new(HashMap::new())
    }).lock().unwrap().insert(name.to_string(), f);
}

pub fn call(name: &str) {
    let registry = REGISTRY.get_or_init(|| {
        Mutex::new(HashMap::new())
    }).lock().unwrap();
    let f = registry.get(name).unwrap();
    f.call::<_, Value>(()).unwrap();
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("tasks");
    module.function(["register"], register)?;
    Ok(module)
}