use rhai::{Engine, Module};
use super::RhaiResult;

pub fn current_branch() -> RhaiResult<String> {
    Ok(git2::Repository::open_from_env().unwrap().head().unwrap().name().unwrap().to_string())
}

pub fn register(engine: &mut Engine) {
    let mut module = Module::new();
    module.set_native_fn("current_branch", current_branch);
    engine.register_static_module("git", module.into());
}