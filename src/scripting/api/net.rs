use crate::net;
use rune::{Any, ContextError, Module};
use std::collections::HashMap;
use std::path::Path;

#[derive(Any)]
struct Client {
    headers: HashMap<String, String>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    pub fn header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn download_file(&self, url: &str, file: &str) -> rune::Result<()> {
        net::download_file(url, Path::new(file), &self.headers)
    }

    pub fn upload_file(&self, url: &str, file: &str) -> rune::Result<()> {
        net::upload_file(url, Path::new(file), &self.headers)
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("net");
    module.ty::<Client>()?;
    module.function(["Client", "new"], Client::new)?;
    module.inst_fn("header", Client::header)?;
    module.inst_fn("download_file", Client::download_file)?;
    module.inst_fn("upload_file", Client::upload_file)?;
    Ok(module)
}
