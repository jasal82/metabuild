use crate::net;
use rune::{ContextError, Module};
use std::path::Path;

pub fn get_file(url: &str, file: &str) -> rune::Result<()> {
    net::download_file(url, Path::new(file))
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("net");
    module.function(["get_file"], get_file)?;
    Ok(module)
}