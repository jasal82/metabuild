use colored::Colorize;
use koto::prelude::*;

fn write_colored(color: &str, style: &str, s: &str) {
    let colored = match style {
        "bold" => s.bold(),
        "dimmed" => s.dimmed(),
        "italic" => s.italic(),
        "underline" => s.underline(),
        "blink" => s.blink(),
        "reversed" => s.reversed(),
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
        "reversed" => s.reversed(),
        "hidden" => s.hidden(),
        "strikethrough" => s.strikethrough(),
        _ => s.normal(),
    }
    .color(color);
    println!("{colored}");
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("sys");

    result.add_fn("is_windows", |_| Ok(cfg!(target_os = "windows").into()));
    result.add_fn("is_mingw", |_| {
        Ok((cfg!(target_os = "windows") && std::env::vars().any(|(k, _)| k == "MSYSTEM")).into())
    });
    result.add_fn("is_linux", |_| Ok(cfg!(target_os = "linux").into()));
    result.add_fn("args", |_| {
        let list = std::env::args()
            .map(|a| KValue::Str(a.as_str().into()))
            .collect();
        Ok(KValue::List(KList::with_data(list)))
    });
    result.add_fn("env", |_| {
        let map = KMap::with_capacity(std::env::vars().count());
        for (k, v) in std::env::vars() {
            map.insert(KString::from(k), KValue::Str(v.as_str().into()));
        }
        Ok(KValue::Map(map))
    });
    result.add_fn("write_colored", |ctx| match ctx.args() {
        [KValue::Str(color), KValue::Str(style), KValue::Str(s)] => {
            write_colored(color, style, s);
            Ok(KValue::Null)
        }
        unexpected => {
            type_error_with_slice("(color: string, style: string, s: string)", unexpected)
        }
    });
    result.add_fn("writeln_colored", |ctx| match ctx.args() {
        [KValue::Str(color), KValue::Str(style), KValue::Str(s)] => {
            writeln_colored(color, style, s);
            Ok(KValue::Null)
        }
        unexpected => {
            type_error_with_slice("(color: string, style: string, s: string)", unexpected)
        }
    });

    result
}
