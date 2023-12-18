use base64::{engine::general_purpose, Engine as _};
use colored::{Color, ColoredString, Colorize};
use koto::prelude::*;
use koto::runtime::Result;
use koto_serialize::SerializableValue;
use tera::{Context, Tera};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Painter {
    text: ColoredString,
}

impl Painter {
    pub fn black(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::Black),
        }
        .into())
    }

    pub fn red(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::Red),
        }
        .into())
    }

    pub fn green(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::Green),
        }
        .into())
    }

    pub fn yellow(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::Yellow),
        }
        .into())
    }

    pub fn blue(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::Blue),
        }
        .into())
    }

    pub fn magenta(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::Magenta),
        }
        .into())
    }

    pub fn cyan(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::Cyan),
        }
        .into())
    }

    pub fn white(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::White),
        }
        .into())
    }

    pub fn bright_black(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::BrightBlack),
        }
        .into())
    }

    pub fn bright_red(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::BrightRed),
        }
        .into())
    }

    pub fn bright_green(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::BrightGreen),
        }
        .into())
    }

    pub fn bright_yellow(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::BrightYellow),
        }
        .into())
    }

    pub fn bright_blue(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::BrightBlue),
        }
        .into())
    }

    pub fn bright_magenta(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::BrightMagenta),
        }
        .into())
    }

    pub fn bright_cyan(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::BrightCyan),
        }
        .into())
    }

    pub fn bright_white(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.color(Color::BrightWhite),
        }
        .into())
    }

    pub fn on_black(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_black(),
        }
        .into())
    }

    pub fn on_red(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_red(),
        }
        .into())
    }

    pub fn on_green(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_green(),
        }
        .into())
    }

    pub fn on_yellow(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_yellow(),
        }
        .into())
    }

    pub fn on_blue(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_blue(),
        }
        .into())
    }

    pub fn on_magenta(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_magenta(),
        }
        .into())
    }

    pub fn on_cyan(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_cyan(),
        }
        .into())
    }

    pub fn on_white(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_white(),
        }
        .into())
    }

    pub fn on_bright_black(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_bright_black(),
        }
        .into())
    }

    pub fn on_bright_red(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_bright_red(),
        }
        .into())
    }

    pub fn on_bright_green(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_bright_green(),
        }
        .into())
    }

    pub fn on_bright_yellow(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_bright_yellow(),
        }
        .into())
    }

    pub fn on_bright_blue(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_bright_blue(),
        }
        .into())
    }

    pub fn on_bright_magenta(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_bright_magenta(),
        }
        .into())
    }

    pub fn on_bright_cyan(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_bright_cyan(),
        }
        .into())
    }

    pub fn on_bright_white(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.on_bright_white(),
        }
        .into())
    }

    pub fn clear(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.clear(),
        }
        .into())
    }

    pub fn normal(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.normal(),
        }
        .into())
    }

    pub fn bold(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.bold(),
        }
        .into())
    }

    pub fn dimmed(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.dimmed(),
        }
        .into())
    }

    pub fn italic(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.italic(),
        }
        .into())
    }

    pub fn underline(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.underline(),
        }
        .into())
    }

    pub fn blink(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.blink(),
        }
        .into())
    }

    pub fn reversed(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.reversed(),
        }
        .into())
    }

    pub fn hidden(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.hidden(),
        }
        .into())
    }

    pub fn strikethrough(self) -> Result<Value> {
        Ok(Painter {
            text: self.text.strikethrough(),
        }
        .into())
    }
}

impl KotoType for Painter {
    const TYPE: &'static str = "Painter";
}

impl KotoObject for Painter {
    fn object_type(&self) -> KString {
        PAINTER_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        KObject::from(Painter {
            text: self.text.clone(),
        })
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        PAINTER_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append(self.text.to_string());
        Ok(())
    }
}

impl From<Painter> for Value {
    fn from(painter: Painter) -> Self {
        KObject::from(painter).into()
    }
}

fn make_painter_entries() -> ValueMap {
    ObjectEntryBuilder::<Painter>::new()
        .method("black", |ctx| {
            Ok(ctx.instance()?.to_owned().black()?.into())
        })
        .method("red", |ctx| Ok(ctx.instance()?.to_owned().red()?.into()))
        .method("green", |ctx| {
            Ok(ctx.instance()?.to_owned().green()?.into())
        })
        .method("yellow", |ctx| {
            Ok(ctx.instance()?.to_owned().yellow()?.into())
        })
        .method("blue", |ctx| Ok(ctx.instance()?.to_owned().blue()?.into()))
        .method("magenta", |ctx| {
            Ok(ctx.instance()?.to_owned().magenta()?.into())
        })
        .method("cyan", |ctx| Ok(ctx.instance()?.to_owned().cyan()?.into()))
        .method("white", |ctx| {
            Ok(ctx.instance()?.to_owned().white()?.into())
        })
        .method("bright_black", |ctx| {
            Ok(ctx.instance()?.to_owned().bright_black()?.into())
        })
        .method("bright_red", |ctx| {
            Ok(ctx.instance()?.to_owned().bright_red()?.into())
        })
        .method("bright_green", |ctx| {
            Ok(ctx.instance()?.to_owned().bright_green()?.into())
        })
        .method("bright_yellow", |ctx| {
            Ok(ctx.instance()?.to_owned().bright_yellow()?.into())
        })
        .method("bright_blue", |ctx| {
            Ok(ctx.instance()?.to_owned().bright_blue()?.into())
        })
        .method("bright_magenta", |ctx| {
            Ok(ctx.instance()?.to_owned().bright_magenta()?.into())
        })
        .method("bright_cyan", |ctx| {
            Ok(ctx.instance()?.to_owned().bright_cyan()?.into())
        })
        .method("bright_white", |ctx| {
            Ok(ctx.instance()?.to_owned().bright_white()?.into())
        })
        .method("on_black", |ctx| {
            Ok(ctx.instance()?.to_owned().on_black()?.into())
        })
        .method("on_red", |ctx| {
            Ok(ctx.instance()?.to_owned().on_red()?.into())
        })
        .method("on_green", |ctx| {
            Ok(ctx.instance()?.to_owned().on_green()?.into())
        })
        .method("on_yellow", |ctx| {
            Ok(ctx.instance()?.to_owned().on_yellow()?.into())
        })
        .method("on_blue", |ctx| {
            Ok(ctx.instance()?.to_owned().on_blue()?.into())
        })
        .method("on_magenta", |ctx| {
            Ok(ctx.instance()?.to_owned().on_magenta()?.into())
        })
        .method("on_cyan", |ctx| {
            Ok(ctx.instance()?.to_owned().on_cyan()?.into())
        })
        .method("on_white", |ctx| {
            Ok(ctx.instance()?.to_owned().on_white()?.into())
        })
        .method("on_bright_black", |ctx| {
            Ok(ctx.instance()?.to_owned().on_bright_black()?.into())
        })
        .method("on_bright_red", |ctx| {
            Ok(ctx.instance()?.to_owned().on_bright_red()?.into())
        })
        .method("on_bright_green", |ctx| {
            Ok(ctx.instance()?.to_owned().on_bright_green()?.into())
        })
        .method("on_bright_yellow", |ctx| {
            Ok(ctx.instance()?.to_owned().on_bright_yellow()?.into())
        })
        .method("on_bright_blue", |ctx| {
            Ok(ctx.instance()?.to_owned().on_bright_blue()?.into())
        })
        .method("on_bright_magenta", |ctx| {
            Ok(ctx.instance()?.to_owned().on_bright_magenta()?.into())
        })
        .method("on_bright_cyan", |ctx| {
            Ok(ctx.instance()?.to_owned().on_bright_cyan()?.into())
        })
        .method("on_bright_white", |ctx| {
            Ok(ctx.instance()?.to_owned().on_bright_white()?.into())
        })
        .method("clear", |ctx| {
            Ok(ctx.instance()?.to_owned().clear()?.into())
        })
        .method("normal", |ctx| {
            Ok(ctx.instance()?.to_owned().normal()?.into())
        })
        .method("bold", |ctx| Ok(ctx.instance()?.to_owned().bold()?.into()))
        .method("dimmed", |ctx| {
            Ok(ctx.instance()?.to_owned().dimmed()?.into())
        })
        .method("italic", |ctx| {
            Ok(ctx.instance()?.to_owned().italic()?.into())
        })
        .method("underline", |ctx| {
            Ok(ctx.instance()?.to_owned().underline()?.into())
        })
        .method("blink", |ctx| {
            Ok(ctx.instance()?.to_owned().blink()?.into())
        })
        .method("reversed", |ctx| {
            Ok(ctx.instance()?.to_owned().reversed()?.into())
        })
        .method("hidden", |ctx| {
            Ok(ctx.instance()?.to_owned().hidden()?.into())
        })
        .method("strikethrough", |ctx| {
            Ok(ctx.instance()?.to_owned().strikethrough()?.into())
        })
        .method("to_string", |ctx| {
            Ok(ctx.instance()?.to_owned().text.to_string().into())
        })
        .build()
}

thread_local! {
    static PAINTER_TYPE_STRING: KString = Painter::TYPE.into();
    static PAINTER_ENTRIES: ValueMap = make_painter_entries();
}

pub fn paint(text: &str) -> Result<Value> {
    Ok(Painter { text: text.into() }.into())
}

enum Source {
    File(String),
    Template(String),
}

fn template_internal(tpl: Source, context: HashMap<String, &Value>) -> Result<Value> {
    let mut tera = Tera::default();
    match tpl {
        Source::File(file) => tera.add_template_file(file, Some("template")).map_err(|e| make_runtime_error!(format!("Could not read template from file: {e}")))?,
        Source::Template(tpl) => tera.add_raw_template("template", tpl.as_str()).map_err(|e| make_runtime_error!(format!("Could not read template from string: {e}")))?,
    }
    let mut tera_context = Context::new();
    for (k, v) in context {
        tera_context.insert(k, &SerializableValue(v));
    }
    tera.render("template", &tera_context).map(|v| v.into()).map_err(|e| make_runtime_error!(format!("Failed to render template: {e}")))
}

pub fn template_file(tpl_file: &str, context: HashMap<String, &Value>) -> Result<Value> {
    template_internal(Source::File(tpl_file.to_string()), context)
}

pub fn template(tpl: &str, context: HashMap<String, &Value>) -> Result<Value> {
    template_internal(Source::Template(tpl.to_string()), context)
}

pub fn encode_string_base64(text: &KString) -> Result<Value> {
    Ok(general_purpose::STANDARD.encode(text.as_str()).into())
}

pub fn encode_bytes_base64(bytes: &KTuple) -> Result<Value> {
    let mut v = Vec::<u8>::new();
    for i in bytes.iter() {
        if let Value::Number(i) = i {
            v.push(u32::try_from(i).expect("Failed to convert byte value").try_into().expect("Failed to convert byte value"))
        } else {
            panic!("Expected a tuple of bytes")
        }
    }
    Ok(general_purpose::STANDARD.encode(v).into())
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("utils");
    result.add_fn("paint", |ctx| match ctx.args() {
        [Value::Str(text)] => paint(text),
        unexpected => type_error_with_slice("(text: string)", unexpected),
    });
    result.add_fn("template", |ctx| match ctx.args() {
        [Value::Str(tpl), Value::Map(context)] => {
            let mut context_map = HashMap::new();
            let items = context.data();
            for (k, v) in items.iter() {
                context_map.insert(k.to_string(), v);
            }
            template(tpl, context_map)
        }
        unexpected => type_error_with_slice("(tpl: string, context: map)", unexpected),
    });
    result.add_fn("template_file", |ctx| match ctx.args() {
        [Value::Str(tpl_file), Value::Map(context)] => {
            let mut context_map = HashMap::new();
            let items = context.data();
            for (k, v) in items.iter() {
                context_map.insert(k.to_string(), v);
            }
            template_file(tpl_file, context_map)
        }
        unexpected => type_error_with_slice("(tpl_file: string, context: map)", unexpected),
    });
    result.add_fn("encode_base64", |ctx| match ctx.args() {
        [Value::Str(text)] => encode_string_base64(text),
        [Value::Tuple(bytes)] => encode_bytes_base64(bytes),
        unexpected => type_error_with_slice("a string or a tuple of bytes", unexpected),
    });
    result.add_fn("decode_base64", |ctx| match ctx.args() {
        [Value::Str(text)] => {
            let decoded = general_purpose::STANDARD.decode(text.as_str()).expect("Failed to decode bytes");
            Ok(KTuple::from(decoded.iter().map(|v| Value::Number(KNumber::from(v))).collect::<Vec<Value>>()).into())
        }
        unexpected => type_error_with_slice("(text: string)", unexpected),
    });

    result
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_template() {}
}
