use koto::prelude::*;
use koto::runtime::Result;
use std::rc::Rc;

pub fn current_branch() -> Result<Value> {
    match git2::Repository::open_from_env() {
        Ok(repo) => {
            match repo.head() {
                Ok(head) => {
                    let branch = head.shorthand().unwrap_or("unknown");
                    Ok(branch.to_string().into())
                }
                Err(_) => Ok("unknown".into()),
            }
        }
        Err(_) => Ok("unknown".into()),
    }
}

#[derive(Clone)]
pub struct Repository {
    inner: Rc<git2::Repository>
}

impl Repository {
    pub fn open(path: &str) -> Result<Value> {
        git2::Repository::open(path)
            .map(|repo| Self { inner: Rc::new(repo) }.into())
            .map_err(|e| make_runtime_error!(format!("Failed to open repository: {e}")))
    }

    pub fn branches(&self) -> Result<Value> {
        let mut branches = Vec::new();
        let repo = &self.inner;
        let branches_iter = repo.branches(None).map_err(|e| make_runtime_error!(format!("Failed to list branches: {e}")))?;
        for branch in branches_iter {
            let (branch, _) = branch.map_err(|e| make_runtime_error!(format!("Failed to get branch info: {e}")))?;
            let branch = branch.name().map_err(|e| make_runtime_error!(format!("Failed to get branch name: {e}")))?;
            if let Some(b) = branch {
                branches.push(b.to_string());
            }
        }
        
        Ok(Value::List(KList::with_data(branches.iter().map(|s| Value::Str(s.clone().into())).collect())))
    }

    pub fn tags(&self) -> Result<Value> {
        let mut tags = Vec::new();
        let repo = &self.inner;
        let ts = repo.tag_names(None).map_err(|e| make_runtime_error!(format!("Failed to list tags: {e}")))?;
        for tag in ts.iter() {
            if let Some(t) = tag {
                tags.push(t.to_string());
            }
        }
        
        Ok(Value::List(KList::with_data(tags.iter().map(|s| Value::Str(s.clone().into())).collect())))
    
    }
}

impl KotoType for Repository {
    const TYPE: &'static str = "Repository";
}

impl KotoObject for Repository {
    fn object_type(&self) -> KString {
        REPOSITORY_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        KObject::from(Repository {
            inner: self.inner.clone(),
        })
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        REPOSITORY_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append(format!("Repository: {}", self.inner.path().display()));
        Ok(())
    }
}

impl From<Repository> for Value {
    fn from(repo: Repository) -> Self {
        KObject::from(repo).into()
    }
}

fn make_repository_entries() -> ValueMap {
    ObjectEntryBuilder::<Repository>::new()
        .method("branches", |ctx| match ctx.args {
            [] => ctx.instance()?.branches(),
            unexpected => type_error_with_slice("()", unexpected),
        })
        .method("tags", |ctx| match ctx.args {
            [] => ctx.instance()?.tags(),
            unexpected => type_error_with_slice("()", unexpected),
        })
        .build()
}

thread_local! {
    static REPOSITORY_TYPE_STRING: KString = KString::from("Repository");
    static REPOSITORY_ENTRIES: ValueMap = make_repository_entries();
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("git");
    result.add_fn("current_branch", |ctx: &mut CallContext<'_>| match ctx.args() {
        [] => current_branch(),
        unexpected => type_error_with_slice("()", unexpected),
    });
    result.add_fn("open", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(path)] => Repository::open(path),
        unexpected => type_error_with_slice("(path: string)", unexpected),
    });

    result
}