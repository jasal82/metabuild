use colored::Colorize;
use rune::runtime::Object;
use rune::{ContextError, Module};

pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

pub fn args() -> Vec<String> {
    std::env::args().collect()
}

pub fn env() -> Object {
    let mut map = Object::new();
    std::env::vars().for_each(|(k, v)| {
        map.insert(k, v.into());
    });
    map
}

pub fn write(s: &str) {
    print!("{s}");
}

pub fn writeln(s: &str) {
    println!("{s}");
}

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
    let mut module = Module::with_crate("sys");
    module.function(["is_windows"], is_windows)?;
    module.function(["is_linux"], is_linux)?;
    module.function(["args"], args)?;
    module.function(["env"], env)?;
    module.function(["write"], write)?;
    module.function(["writeln"], writeln)?;
    module.function(["write_colored"], write_colored)?;
    module.function(["writeln_colored"], writeln_colored)?;
    Ok(module)
}
