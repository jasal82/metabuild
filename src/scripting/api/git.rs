use rune::{Any, ContextError, Module, vm_write, runtime::VmResult, runtime::Formatter};
use rune::alloc::fmt::TryWrite;

#[rune::function]
pub fn current_branch() -> VmResult<String> {
    match git2::Repository::open_from_env() {
        Ok(repo) => {
            match repo.head() {
                Ok(head) => {
                    let branch = head.shorthand().unwrap_or("unknown");
                    VmResult::Ok(branch.to_string())
                }
                Err(_) => VmResult::Ok("unknown".to_string()),
            }
        }
        Err(_) => VmResult::Ok("unknown".to_string()),
    }
}

#[derive(Any)]
#[rune(item = ::git)]
pub struct Repository {
    inner: git2::Repository
}

#[derive(Any)]
#[rune(item = ::git)]
pub struct Error {
    error: git2::Error
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

impl From<git2::Error> for Error {
    fn from(error: git2::Error) -> Self {
        Self { error }
    }
}

impl Repository {
    #[rune::function(path = Self::open)]
    pub fn open(path: &str) -> Result<Self, Error> {
        git2::Repository::open(path)
            .map(|repo| Self { inner: repo })
            .map_err(|e| e.into())
    }

    #[rune::function]
    pub fn branches(&self) -> Result<Vec<String>, Error> {
        let mut branches = Vec::new();
        let repo = &self.inner;
        let branches_iter = repo.branches(None).map_err(|e| Error{error: e})?;
        for branch in branches_iter {
            let (branch, _) = branch.map_err(|e| Error{error: e})?;
            let branch = branch.name().map_err(|e| Error{error: e})?;
            branches.push(branch.unwrap().to_string());
        }
        Ok(branches)
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("git")?;
    module.ty::<Repository>()?;
    module.ty::<Error>()?;
    module.function_meta(Error::display)?;
    module.function_meta(Error::debug)?;
    module.function_meta(Repository::open)?;
    module.function_meta(Repository::branches)?;
    module.function_meta(current_branch)?;
    Ok(module)
}
