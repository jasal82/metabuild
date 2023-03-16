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

pub fn write(s: &str) -> RhaiResult<()> {
    print!("{}", s);
    Ok(())
}

pub fn writeln(s: &str) -> RhaiResult<()> {
    println!("{}", s);
    Ok(())
}

pub fn write_colored(color: &str, style: &str, s: &str) -> RhaiResult<()> {
    let colored = match style {
        "bold" => s.bold(),
        "dimmed" => s.dimmed(),
        "italic" => s.italic(),
        "underline" => s.underline(),
        "blink" => s.blink(),
        "reverse" => s.reverse(),
        "hidden" => s.hidden(),
        "strikethrough" => s.strikethrough(),
        _ => s.normal(),
    }.color(color);
    print!("{}", colored);
    Ok(())
}

pub fn writeln_colored(color: &str, style: &str, s: &str) -> RhaiResult<()> {
    let colored = match style {
        "bold" => s.bold(),
        "dimmed" => s.dimmed(),
        "italic" => s.italic(),
        "underline" => s.underline(),
        "blink" => s.blink(),
        "reverse" => s.reverse(),
        "hidden" => s.hidden(),
        "strikethrough" => s.strikethrough(),
        _ => s.normal(),
    }.color(color);
    println!("{}", colored);
    Ok(())
}

pub fn register(engine: &mut Engine) {
    let mut module = Module::new();
    module.set_native_fn("is_windows", is_windows);
    module.set_native_fn("is_linux", is_linux);
    module.set_native_fn("args", args);
    module.set_native_fn("env", env);
    module.set_native_fn("write", write);
    module.set_native_fn("writeln", writeln);
    module.set_native_fn("write_colored", write_colored);
    module.set_native_fn("writeln_colored", writeln_colored);
    engine.register_static_module("sys", module.into());
}