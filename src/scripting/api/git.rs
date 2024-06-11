use koto::{derive::*, prelude::*, Result};
use std::rc::Rc;

pub fn current_branch() -> Result<KValue> {
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

#[derive(Clone, KotoCopy, KotoType)]
pub struct Repository {
    inner: Rc<git2::Repository>
}

#[koto_impl]
impl Repository {
    pub fn open(path: &str) -> Result<KValue> {
        git2::Repository::open(path)
            .map(|repo| Self { inner: Rc::new(repo) }.into())
            .map_err(|e| koto::runtime::Error::from(format!("Failed to open repository: {e}")))
            .into()
    }

    #[koto_method]
    pub fn branches(&self) -> Result<KValue> {
        let mut branches = Vec::new();
        let repo = &self.inner;
        let branches_iter = repo.branches(None).map_err(|e| koto::runtime::Error::from(format!("Failed to list branches: {e}")))?;
        for branch in branches_iter {
            let (branch, _) = branch.map_err(|e| koto::runtime::Error::from(format!("Failed to get branch info: {e}")))?;
            let branch = branch.name().map_err(|e| koto::runtime::Error::from(format!("Failed to get branch name: {e}")))?;
            if let Some(b) = branch {
                branches.push(b.to_string());
            }
        }
        
        Ok(KValue::List(KList::with_data(branches.iter().map(|s| KValue::Str(s.clone().into())).collect())))
    }

    #[koto_method]
    pub fn tags(&self) -> Result<KValue> {
        let mut tags = Vec::new();
        let repo = &self.inner;
        let ts = repo.tag_names(None).map_err(|e| koto::runtime::Error::from(format!("Failed to list tags: {e}")))?;
        for tag in ts.iter() {
            if let Some(t) = tag {
                tags.push(t.to_string());
            }
        }
        
        Ok(KValue::List(KList::with_data(tags.iter().map(|s| KValue::Str(s.clone().into())).collect())))
    
    }
}

impl KotoObject for Repository {
    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append(format!("Repository: {}", self.inner.path().display()));
        Ok(())
    }
}

impl From<Repository> for KValue {
    fn from(repo: Repository) -> Self {
        KObject::from(repo).into()
    }
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("git");
    result.add_fn("current_branch", |ctx: &mut CallContext<'_>| match ctx.args() {
        [] => current_branch(),
        unexpected => type_error_with_slice("()", unexpected),
    });
    result.add_fn("open", |ctx: &mut CallContext<'_>| match ctx.args() {
        [KValue::Str(path)] => Repository::open(path),
        unexpected => type_error_with_slice("(path: string)", unexpected),
    });

    result
}