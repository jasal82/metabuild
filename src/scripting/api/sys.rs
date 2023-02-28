use colored::Colorize;
use rhai::{Engine, Dynamic, Map, Module};
use super::RhaiResult;

pub fn is_windows() -> RhaiResult<bool> {
    Ok(cfg!(target_os = "windows"))
}

pub fn is_linux() -> RhaiResult<bool> {
    Ok(cfg!(target_os = "linux"))
}

pub fn args() -> RhaiResult<rhai::Array> {
    Ok(std::env::args().map(|i| Dynamic::from(i)).collect())
}

pub fn env() -> RhaiResult<Map> {
    let mut map = Map::new();
    std::env::vars().for_each(|(k, v)| {
        map.insert(k.into(), v.into());
    });
    Ok(map)
}

pub fn print_colored(s: &str, color: &str) -> RhaiResult<()> {
    print!("{}", s.color(color));
    Ok(())
}

pub fn println_colored(s: &str, color: &str) -> RhaiResult<()> {
    println!("{}", s.color(color));
    Ok(())
}

pub fn register(engine: &mut Engine) {
    let mut module = Module::new();
    module.set_native_fn("is_windows", is_windows);
    module.set_native_fn("is_linux", is_linux);
    module.set_native_fn("args", args);
    module.set_native_fn("env", env);
    module.set_native_fn("print_colored", print_colored);
    module.set_native_fn("println_colored", println_colored);
    engine.register_static_module("sys", module.into());
}