use crate::net;
use rune::{Any, ContextError, Module};
use std::collections::HashMap;
use std::path::Path;

#[derive(Any)]
#[rune(item = ::net)]
struct Client {
    headers: HashMap<String, String>,
}

impl Client {
    #[rune::function(path = Self::new)]
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    #[rune::function]
    pub fn header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    #[rune::function]
    pub fn download_file(&self, url: &str, file: &str) -> Result<(), anyhow::Error> {
        net::download_file(url, Path::new(file), &self.headers)
    }

    #[rune::function]
    pub fn upload_file(&self, url: &str, file: &str) -> Result<(), anyhow::Error> {
        net::upload_file(url, Path::new(file), &self.headers)
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("net")?;
    module.ty::<Client>()?;

    module.function_meta(Client::new)?;
    module.function_meta(Client::header)?;
    module.function_meta(Client::download_file)?;
    module.function_meta(Client::upload_file)?;

    Ok(module)
}
