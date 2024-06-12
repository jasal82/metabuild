use base64::{engine::general_purpose, Engine as _};
use colored::{ColoredString, Colorize};
use koto::{derive::*, prelude::*, Result};
use koto_serialize::SerializableValue;
use std::collections::HashMap;
use tera::{Context, Tera};

#[derive(Clone, Debug, KotoCopy, KotoType)]
pub struct Painter {
    text: Option<ColoredString>,
}

#[koto_impl]
impl Painter {
    #[koto_method]
    pub fn black(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.black()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn red(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.red()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn green(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.green()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn yellow(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.yellow()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn blue(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.blue()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn magenta(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.magenta()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn cyan(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.cyan()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn white(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.white()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn bright_black(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.bright_black()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn bright_red(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.bright_red()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn bright_green(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.bright_green()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn bright_yellow(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.bright_yellow()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn bright_blue(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.bright_blue()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn bright_magenta(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.bright_magenta()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn bright_cyan(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.bright_cyan()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn bright_white(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.bright_white()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_black(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_black()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_red(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_red()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_green(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_green()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_yellow(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_yellow()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_blue(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_blue()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_magenta(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_magenta()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_cyan(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_cyan()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_white(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_white()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_bright_black(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_bright_black()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_bright_red(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_bright_red()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_bright_green(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_bright_green()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_bright_yellow(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_bright_yellow()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_bright_blue(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_bright_blue()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_bright_magenta(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_bright_magenta()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_bright_cyan(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_bright_cyan()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn on_bright_white(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.on_bright_white()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn clear(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.clear()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn normal(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.normal()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn bold(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.bold()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn dimmed(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.dimmed()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn italic(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.italic()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn underline(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.underline()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn blink(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.blink()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn reversed(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.reversed()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn hidden(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.hidden()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn strikethrough(ctx: MethodContext<Self>) -> Result<KValue> {
        ctx.instance_mut()?.text = ctx.instance_mut()?.text.take().and_then(|t| Some(t.strikethrough()));
        ctx.instance_result()
    }

    #[koto_method]
    pub fn to_string(&self) -> Result<KValue> {
        Ok(self.text.as_ref().unwrap().to_string().into())
    }
}

impl KotoObject for Painter {
    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        // This implementation is used to implicitly render the Painter
        // instance in Koto, so keep the format expression exactly as-is.
        ctx.append(format!("{}", self.text.as_ref().unwrap().to_string()));
        Ok(())
    }
}

impl From<Painter> for KValue {
    fn from(painter: Painter) -> Self {
        KObject::from(painter).into()
    }
}

pub fn paint(text: &str) -> Result<KValue> {
    Ok(Painter { text: Some(text.into()) }.into())
}

enum Source {
    File(String),
    Template(String),
}

fn template_internal(tpl: Source, context: HashMap<String, &KValue>) -> Result<KValue> {
    let mut tera = Tera::default();
    match tpl {
        Source::File(file) => tera
            .add_template_file(file, Some("template"))
            .map_err(|e| koto::runtime::Error::from(format!("Could not read template from file: {e}")))?,
        Source::Template(tpl) => tera
            .add_raw_template("template", tpl.as_str())
            .map_err(|e| {
                koto::runtime::Error::from(format!("Could not read template from string: {e}"))
            })?,
    }
    let mut tera_context = Context::new();
    for (k, v) in context {
        tera_context.insert(k, &SerializableValue(v));
    }
    let result = tera.render("template", &tera_context)
        .map_err(|e| koto::runtime::Error::from(format!("Failed to render template: {e}")))?;
    Ok(result.into())
}

pub fn template_file(tpl_file: &str, context: HashMap<String, &KValue>) -> Result<KValue> {
    template_internal(Source::File(tpl_file.to_string()), context)
}

pub fn template(tpl: &str, context: HashMap<String, &KValue>) -> Result<KValue> {
    template_internal(Source::Template(tpl.to_string()), context)
}

pub fn encode_string_base64(text: &KString) -> Result<KValue> {
    Ok(general_purpose::STANDARD.encode(text.as_str()).into())
}

pub fn encode_bytes_base64(bytes: &KTuple) -> Result<KValue> {
    let mut v = Vec::<u8>::new();
    for i in bytes.iter() {
        if let KValue::Number(i) = i {
            v.push(
                u32::try_from(i)
                    .expect("Failed to convert byte value")
                    .try_into()
                    .expect("Failed to convert byte value"),
            )
        } else {
            panic!("Expected a tuple of bytes")
        }
    }
    Ok(general_purpose::STANDARD.encode(v).into())
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("utils");
    result.add_fn("paint", |ctx| match ctx.args() {
        [KValue::Str(text)] => paint(text),
        unexpected => type_error_with_slice("(text: string)", unexpected),
    });
    result.add_fn("template", |ctx| match ctx.args() {
        [KValue::Str(tpl), KValue::Map(context)] => {
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
        [KValue::Str(tpl_file), KValue::Map(context)] => {
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
        [KValue::Str(text)] => encode_string_base64(text),
        [KValue::Tuple(bytes)] => encode_bytes_base64(bytes),
        unexpected => type_error_with_slice("a string or a tuple of bytes", unexpected),
    });
    result.add_fn("decode_base64", |ctx| match ctx.args() {
        [KValue::Str(text)] => {
            let decoded = general_purpose::STANDARD
                .decode(text.as_str())
                .expect("Failed to decode bytes");
            Ok(KTuple::from(
                decoded
                    .iter()
                    .map(|v| KValue::Number(KNumber::from(v)))
                    .collect::<Vec<KValue>>(),
            )
            .into())
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
