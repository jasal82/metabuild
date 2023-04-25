use rune::ast::Span;
use rune::termcolor::{ColorChoice, StandardStream};
use rune::{
    compile::{CompileError, FileSourceLoader, Item, SourceLoader},
    Context, Diagnostics, Source, Sources, Vm,
};
use std::path::Path;
use std::sync::Arc;

mod api;

#[derive(Default)]
pub struct CustomSourceLoader {
    file_loader: FileSourceLoader,
}

impl CustomSourceLoader {
    pub fn new() -> Self {
        Self {
            file_loader: FileSourceLoader::new(),
        }
    }
}

impl SourceLoader for CustomSourceLoader {
    fn load(&mut self, root: &Path, item: &Item, span: Span) -> Result<Source, CompileError> {
        // Try the default loader first. If that fails, try to use the module
        // path as root instead. We have to add a dummy file name to the path
        // because the FileSourceLoader pops the last path component off.
        match self.file_loader.load(root, item, span) {
            Ok(source) => Ok(source),
            Err(_) => {
                let module_path = Path::new(".mb/modules/dummy");
                self.file_loader.load(module_path, item, span)
            }
        }
    }
}

pub fn run_tasks(script_file: &Path, tasks: &[String], warn: bool) -> Result<(), anyhow::Error> {
    let mut context = Context::with_default_modules()?;
    context.install(&rune_modules::json::module(true)?)?;
    context.install(&api::arch::module()?)?;
    context.install(&api::cmd::module()?)?;
    context.install(&api::docker::module()?)?;
    context.install(&api::fs::module()?)?;
    context.install(&api::git::module()?)?;
    context.install(&api::http::module()?)?;
    context.install(&api::net::module()?)?;
    context.install(&api::re::module()?)?;
    context.install(&api::str::module()?)?;
    context.install(&api::sys::module()?)?;
    context.install(&api::metabuild::module()?)?;
    context.install(&api::toml::module()?)?;
    context.install(&api::yaml::module()?)?;

    let mut sources = Sources::new();
    sources.insert(Source::from_path(script_file)?);

    let mut diagnostics = match warn {
        true => Diagnostics::new(),
        false => Diagnostics::without_warnings(),
    };

    let mut source_loader = CustomSourceLoader::new();

    let mut options = rune::Options::default();
    options.debug_info(true);

    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .with_source_loader(&mut source_loader)
        .with_options(&options)
        .build();

    if !diagnostics.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        diagnostics.emit(&mut writer, &sources)?;
    }

    let unit = result?;

    let mut vm = Vm::new(Arc::new(context.runtime()), Arc::new(unit));
    let mut execution = vm.execute(
        ["main"],
        (tasks.iter().map(|t| t.to_owned()).collect::<Vec<String>>(),),
    )?;
    let result = execution.complete();
    let _errored = match result {
        Ok(value) => {
            if value.into_unit().is_err() {
                crate::logging::warning("Function main() returned a non-unit value. All errors must be handled in the script.");
            }
            None
        },
        Err(error) => {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            error.emit(&mut writer, &sources)?;
            Some(error)
        }
    };

    Ok(())
}
