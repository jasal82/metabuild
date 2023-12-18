use colored::Colorize;
use koto::prelude::*;

fn write_colored(color: &str, style: &str, s: &str) {
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

fn writeln_colored(color: &str, style: &str, s: &str) {
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

pub fn make_module() -> KMap {
    let result = KMap::with_type("sys");

    result.add_fn("is_windows", |ctx| Ok(cfg!(target_os = "windows").into()));
    result.add_fn("is_mingw", |ctx| {
        Ok((cfg!(target_os = "windows") && std::env::vars().any(|(k, _)| k == "MSYSTEM")).into())
    });
    result.add_fn("is_linux", |ctx| Ok(cfg!(target_os = "linux").into()));
    result.add_fn("args", |ctx| {
        let list = std::env::args()
            .map(|a| Value::Str(a.as_str().into()))
            .collect::<ValueVec>();
        Ok(Value::List(KList::with_data(list)))
    });
    result.add_fn("env", |ctx| {
        let mut map = KMap::with_capacity(std::env::vars().count());
        for (k, v) in std::env::vars() {
            map.add_value(&k, Value::Str(v.as_str().into()));
        }
        Ok(Value::Map(map))
    });
    result.add_fn("write_colored", |ctx| match ctx.args() {
        [Value::Str(color), Value::Str(style), Value::Str(s)] => {
            write_colored(color, style, s);
            Ok(Value::Null)
        }
        unexpected => {
            type_error_with_slice("(color: string, style: string, s: string)", unexpected)
        }
    });
    result.add_fn("writeln_colored", |ctx| match ctx.args() {
        [Value::Str(color), Value::Str(style), Value::Str(s)] => {
            writeln_colored(color, style, s);
            Ok(Value::Null)
        }
        unexpected => {
            type_error_with_slice("(color: string, style: string, s: string)", unexpected)
        }
    });

    result
}
