use colored::Colorize;
use rune::runtime::{Object, Shared, Value};
use rune::{ContextError, Module};

#[rune::function]
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

#[rune::function]
pub fn is_mingw() -> bool {
    cfg!(target_os = "windows") && std::env::vars().any(|(k, _)| k == "MSYSTEM")
}

#[rune::function]
pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

#[rune::function]
pub fn args() -> Vec<String> {
    std::env::args().collect()
}

#[rune::function]
pub fn env() -> Object {
    let mut map = Object::new();
    std::env::vars().for_each(|(k, v)| {
        if let Ok(k) = rune::alloc::String::try_from(k) {
            if let Ok(v) = rune::alloc::String::try_from(v) {
                if let Ok(v) = Shared::new(v) {
                    let _ = map.insert(k, Value::from(v));
                }
            }
        }
    });
    map
}

#[rune::function]
pub fn write(s: &str) {
    print!("{s}");
}

#[rune::function]
pub fn writeln(s: &str) {
    println!("{s}");
}

#[rune::function]
pub fn write_colored(color: &str, style: &str, s: &str) {
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
    }
    .color(color);
    print!("{colored}");
}

#[rune::function]
pub fn writeln_colored(color: &str, style: &str, s: &str) {
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
    }
    .color(color);
    println!("{colored}");
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("sys")?;

    module.function_meta(is_windows)?;
    module.function_meta(is_mingw)?;
    module.function_meta(is_linux)?;
    module.function_meta(args)?;
    module.function_meta(env)?;
    module.function_meta(write)?;
    module.function_meta(writeln)?;
    module.function_meta(write_colored)?;
    module.function_meta(writeln_colored)?;

    Ok(module)
}
