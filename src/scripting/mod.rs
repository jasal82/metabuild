use rune::termcolor::{ColorChoice, StandardStream};
use rune::ast::Span;
use rune::{Context, compile::{CompileError, CompileErrorKind, FileSourceLoader, Item, SourceLoader}, Diagnostics, FromValue, Source, Sources, Unit, runtime::UnitFn, Vm};
use std::path::Path;
use std::sync::Arc;

mod api;

#[derive(Default)]
pub struct CustomSourceLoader {
    file_loader: FileSourceLoader
}

impl CustomSourceLoader {
    pub fn new() -> Self {
        Self {
            file_loader: FileSourceLoader::new()
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

pub fn run_tasks(script_file: &Path, tasks: &Vec<String>) -> Result<(), anyhow::Error> {
    let mut context = Context::with_default_modules()?;
    context.install(&rune_modules::io::module(true)?)?;
    context.install(&rune_modules::http::module(true)?)?;
    context.install(&api::arch::module()?)?;
    context.install(&api::cmd::module()?)?;
    context.install(&api::fs::module()?)?;
    context.install(&api::git::module()?)?;
    context.install(&api::net::module()?)?;
    context.install(&api::re::module()?)?;
    context.install(&api::str::module()?)?;
    context.install(&api::sys::module()?)?;
    context.install(&api::tasks::module()?)?;
    context.install(&api::toml::module()?)?;
    context.install(&api::yaml::module()?)?;

    let mut sources = Sources::new();
    sources.insert(Source::from_path(script_file)?);

    let mut diagnostics = Diagnostics::new();
    let mut source_loader = CustomSourceLoader::new();

    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .with_source_loader(&mut source_loader)
        .build();

    if !diagnostics.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        diagnostics.emit(&mut writer, &sources)?;
    }

    let unit = result?;

    let mut vm = Vm::new(Arc::new(context.runtime()), Arc::new(unit));
    let mut execution = vm.execute(["main"], ())?;
    let result = execution.complete();
    let errored = match result {
        Ok(result) => {
            None
        },
        Err(error) => {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            error.emit(&mut writer, &sources)?;
            Some(error)
        }
    };

    api::tasks::call("blubb");
    //let output = i64::from_value(output)?;
    
    /*if tasks.len() == 0 {
        return Err("No tasks specified".into());
    }
    
    let available_tasks = get_available_tasks(&ast);
    let mut unknown_tasks = Vec::new();
    tasks.iter().for_each(|task| {
        if !available_tasks.contains(&task) {
            unknown_tasks.push(task);
        }
    });

    if unknown_tasks.len() > 0 {
        return Err(format!("Unknown tasks: {:?}", unknown_tasks).into());
    }

    let mut exit_code : u8 = 0;
    for task in tasks {
        let mut scope = Scope::new();
        let result = engine.call_fn::<Dynamic>(&mut scope, &ast, get_task_fn_name(&task), ()).expect("Failed to call task");

        if result.is_bool() {
            exit_code = !result.as_bool().unwrap_or(false) as u8;
        }
    }*/

    Ok(())
}