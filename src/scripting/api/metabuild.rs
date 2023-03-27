use rune::{Any, ContextError, Module};
use rune::runtime::{Function, Key, Value, VmError, Vec};
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
            None => Err(VmError::panic(format!("Function {:?} not found", name.into_value()))),
        }
    }

    pub fn run(&self, tasks: &Vec) {
        for task in tasks {
            match self.tasks.get(&Key::from_value(task).unwrap()) {
                Some(f) => {
                    let _ = f.call::<std::vec::Vec<Value>, Value>(vec![]);
                }
                None => {
                    println!("Task {:?} not found", task);
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