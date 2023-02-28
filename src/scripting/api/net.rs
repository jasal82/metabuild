use crate::net;
use rhai::{Engine, Module};
use std::path::Path;
use super::RhaiResult;

pub fn get_file(url: &str, file: &str) -> RhaiResult<()> {
    net::download_file(url, Path::new(file)).map_err(|_| error!("Failed to download file"))
}

pub fn register(engine: &mut Engine) {
    let mut module = Module::new();
    module.set_native_fn("get_file", get_file);
    engine.register_static_module("net", module.into());
}