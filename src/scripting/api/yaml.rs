use rune::{Any, ContextError, Module, vm_write};
use rune::runtime::{Formatter, Value, VmResult};
use rune::alloc::fmt::TryWrite;
use rune::alloc::String;

#[derive(Any)]
#[rune(item = ::yaml)]
pub struct Error {
    error: serde_yaml::Error,
}

impl Error {
    #[rune::function(protocol = STRING_DISPLAY)]
    fn display(&self, f: &mut Formatter) -> VmResult<()> {
        vm_write!(f, "{}", self.error);
        VmResult::Ok(())
    }

    #[rune::function(protocol = STRING_DEBUG)]
    fn debug(&self, f: &mut Formatter) -> VmResult<()> {
        vm_write!(f, "{:?}", self.error);
        VmResult::Ok(())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Self { error }
    }
}

#[rune::function]
fn from_string(string: &str) -> Result<Value, Error> {
    Ok(serde_yaml::from_str(string)?)
}

#[rune::function(vm_result)]
fn to_string(value: Value) -> Result<String, Error> {
    Ok(String::try_from(serde_yaml::to_string(&value)?).vm?)
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("yaml")?;
    module.ty::<Error>()?;
    module.function_meta(Error::display)?;
    module.function_meta(Error::debug)?;

    module.function_meta(from_string)?;
    module.function_meta(to_string)?;
    Ok(module)
}