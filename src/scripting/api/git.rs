use rune::{ContextError, Module};

pub fn current_branch() -> rune::Result<String> {
    Ok(git2::Repository::open_from_env()?.head()?.name().unwrap_or("unknown").to_string())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("git");
    module.function(["current_branch"], current_branch)?;
    Ok(module)
}