use regex::Regex;
use rhai::{Engine, Module};
use super::RhaiResult;

pub fn matches(pattern: &str, text: &str) -> RhaiResult<bool> {
    let re = Regex::new(pattern).unwrap();
    Ok(re.is_match(text))
}

pub fn replace(pattern: &str, replacement: &str, text: &str) -> RhaiResult<String> {
    let re = Regex::new(pattern).unwrap();
    Ok(re.replace_all(text, replacement).into_owned())
}

pub fn register(engine: &mut Engine) {
    let mut module = Module::new();
    module.set_native_fn("matches", matches);
    module.set_native_fn("replace", replace);
    engine.register_static_module("re", module.into());
}