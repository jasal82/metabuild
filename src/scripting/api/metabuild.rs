use crate::logging::error;
use rune::runtime::{Function, Key, Value, Vec, VmError};
use rune::{Any, ContextError, Module};
use std::collections::HashMap;

#[derive(Any)]
#[rune(item = ::metabuild)]
struct TaskRunner {
    tasks: HashMap<Key, Function>,
}

impl TaskRunner {
    #[rune::function(path = Self::new)]
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    #[rune::function]
    pub fn register(&mut self, name: Key, f: Function) {
        self.tasks.insert(name, f);
    }

    #[rune::function]
    pub fn call(&self, name: Key, args: &Vec) -> Result<Value, VmError> {
        match self.tasks.get(&name) {
            Some(f) => f.call::<std::vec::Vec<Value>, Value>(args.clone().into_inner()),
            None => {
                let v = name.into_value();
                error(format!("Function {v:?} not found"));
                Err(VmError::panic(format!("Function {v:?} not found")))
            }
        }
    }

    #[rune::function]
    pub fn run(&self, tasks: &Vec) {
        if tasks.is_empty() {
            error("No tasks specified");
        } else {
            for task in tasks {
                match self.tasks.get(&Key::from_value(task).unwrap()) {
                    Some(f) => match f.call::<(), Value>(()) {
                        Ok(_) => {}
                        Err(e) => {
                            error(&format!("Task {task:?} failed: {e}"));
                        }
                    },
                    None => {
                        error(&format!("Task {task:?} not found"));
                    }
                }
            }
        }
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("metabuild")?;
    module.ty::<TaskRunner>()?;

    module.function_meta(TaskRunner::new)?;
    module.function_meta(TaskRunner::register)?;
    module.function_meta(TaskRunner::call)?;
    module.function_meta(TaskRunner::run)?;

    Ok(module)
}
