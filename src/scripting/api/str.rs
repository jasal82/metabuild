use rune::{ContextError, Module};
use rune::runtime::Object;
use tera::{Tera, Context};

enum Source {
    File(String),
    Template(String),
}

fn template_internal(tpl: Source, context: Object) -> String {
    let mut tera = Tera::default();
    match tpl {
        Source::File(file) => tera.add_template_file(file, Some("template")).unwrap(),
        Source::Template(tpl) => tera.add_raw_template("template", &tpl).unwrap(),
    }
    let mut tera_context = Context::new();
    for (k, v) in context {
        tera_context.insert(k, v.into_string().unwrap().borrow_ref().unwrap().as_str());
    }
    tera.render("template", &tera_context).unwrap()
}

pub fn template_file(tpl_file: &str, context: Object) -> String {
    template_internal(Source::File(tpl_file.to_string()), context)
}

pub fn template(tpl: &str, context: Object) -> String {
    template_internal(Source::Template(tpl.to_string()), context)
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("str");
    module.function(["template_file"], template_file)?;
    module.function(["template"], template)?;
    Ok(module)
}