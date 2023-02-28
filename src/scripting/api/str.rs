use rhai::{Engine, Map, Module};
use super::RhaiResult;
use tera::{Tera, Context};

enum Source {
    File(String),
    Template(String),
}

fn template_internal(tpl: Source, context: Map) -> String {
    let mut tera = Tera::default();
    match tpl {
        Source::File(file) => tera.add_template_file(file, Some("template")).unwrap(),
        Source::Template(tpl) => tera.add_raw_template("template", &tpl).unwrap(),
    }
    let mut tera_context = Context::new();
    for (k, v) in context {
        tera_context.insert(k, &v.into_string().unwrap());
    }
    tera.render("template", &tera_context).unwrap()
}

pub fn template_file(tpl_file: &str, context: Map) -> RhaiResult<String> {
    Ok(template_internal(Source::File(tpl_file.to_string()), context))
}

pub fn template(tpl: &str, context: Map) -> RhaiResult<String> {
    Ok(template_internal(Source::Template(tpl.to_string()), context))
}

pub fn register(engine: &mut Engine) {
    let mut module = Module::new();
    module.set_native_fn("template_file", template_file);
    module.set_native_fn("template", template);
    engine.register_static_module("str", module.into());
}