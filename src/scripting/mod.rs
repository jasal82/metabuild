use rune::termcolor::{ColorChoice, StandardStream};
use rune::{Context, Diagnostics, FromValue, Source, Sources, Vm};
use std::path::Path;
use std::sync::Arc;

mod api;

/*fn build_engine() -> Engine {
    let mut engine = Engine::new();
    api::arch::register(&mut engine);
    api::cmd::register(&mut engine);
    api::fs::register(&mut engine);
    api::git::register(&mut engine);
    api::net::register(&mut engine);
    api::re::register(&mut engine);
    api::str::register(&mut engine);
    api::sys::register(&mut engine);
    api::toml::register(&mut engine);
    api::yaml::register(&mut engine);
    engine.set_module_resolver(FileModuleResolver::new_with_path(".mb/modules"));
    engine
}

fn get_task_fn_name(task_name: &str) -> String {
    format!("task_{}", task_name)
}

fn get_available_tasks(ast: &rhai::AST) -> Vec<String> {
    let mut tasks : Vec<String> = Vec::new();
    ast.iter_functions().filter(|f| f.name.starts_with("task_")).for_each(|f| {
        tasks.push(f.name[5..].to_string());
    });
    tasks
}*/

pub fn run_tasks(script_file: &Path, tasks: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::with_default_modules()?;
    context.install(&rune_modules::http::module(true)?)?;
    context.install(&rune_modules::process::module(true)?)?;
    context.install(&api::arch::module()?)?;
    context.install(&api::fs::module()?)?;
    context.install(&api::git::module()?)?;
    context.install(&api::net::module()?)?;
    context.install(&api::re::module()?)?;
    context.install(&api::str::module()?)?;
    context.install(&api::sys::module()?)?;
    context.install(&api::toml::module()?)?;
    context.install(&api::yaml::module()?)?;

    let mut sources = Sources::new();
    sources.insert(Source::from_path(script_file)?);

    let mut diagnostics = Diagnostics::new();

    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if !diagnostics.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        diagnostics.emit(&mut writer, &sources)?;
    }

    let unit = result?;

    let mut vm = Vm::new(Arc::new(context.runtime()), Arc::new(unit));
    let output = vm.execute(&["main"], ())?.complete()?;
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