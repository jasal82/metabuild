use crate::net;
use rune::{ContextError, Module};
use std::path::Path;

pub fn download_file(url: &str, file: &str) -> rune::Result<()> {
    net::download_file(url, Path::new(file))
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("net");
    module.function(["download_file"], download_file)?;
    Ok(module)
}