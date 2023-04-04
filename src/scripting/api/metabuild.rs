use crate::logging::error;
use rune::runtime::{Function, Key, Value, Vec, VmError};
use rune::{Any, ContextError, Module};
use std::collections::HashMap;

#[derive(Any)]
struct TaskRunner {
    tasks: HashMap<Key, Function>,
}

impl TaskRunner {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: Key, f: Function) {
        self.tasks.insert(name, f);
    }

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
    let mut module = Module::with_crate("metabuild");
    module.ty::<TaskRunner>()?;
    module.function(["TaskRunner", "new"], TaskRunner::new)?;
    module.inst_fn("register", TaskRunner::register)?;
    module.inst_fn("call", TaskRunner::call)?;
    module.inst_fn("run", TaskRunner::run)?;
    Ok(module)
}
