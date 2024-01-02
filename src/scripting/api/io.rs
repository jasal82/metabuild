use koto::prelude::*;
use koto::runtime::Result;
use path_absolutize::*;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone)]
struct TempDir {
    handle: Rc<tempfile::TempDir>,
    path: Rc<str>,
}

impl TempDir {
    fn new() -> Result<Self> {
        let handle = tempfile::tempdir()
            .map_err(|e| make_runtime_error!(format!("Failed to create tempdir {e}")))?;
        let path = handle
            .path()
            .to_str()
            .expect("Failed to convert path to string")
            .to_string()
            .into();
        Ok(Self {
            handle: Rc::new(handle),
            path,
        })
    }

    fn path(&self) -> &str {
        &self.path
    }
}

impl KotoType for TempDir {
    const TYPE: &'static str = "TempDir";
}

impl KotoObject for TempDir {
    fn object_type(&self) -> KString {
        TEMP_DIR_TYPE_STRING.with(|s| s.clone())
    }

    fn copy(&self) -> KObject {
        KObject::from(TempDir {
            handle: self.handle.clone(),
            path: self.path.clone(),
        })
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        TEMP_DIR_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append(format!("TempDir: {}", self.path));
        Ok(())
    }
}

impl From<TempDir> for Value {
    fn from(temp_dir: TempDir) -> Self {
        KObject::from(temp_dir).into()
    }
}

fn make_temp_dir_entries() -> ValueMap {
    ObjectEntryBuilder::<TempDir>::new()
        .method("path", |ctx| match ctx.args {
            [] => Ok(ctx.instance()?.path().into()),
            unexpected => type_error_with_slice("()", unexpected),
        })
        .build()
}

thread_local! {
    static TEMP_DIR_TYPE_STRING: KString = TempDir::TYPE.into();
    static TEMP_DIR_ENTRIES: ValueMap = make_temp_dir_entries();
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("io_ext");
    result.add_fn("is_file", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(path)] => Ok(Path::new(path.as_str()).is_file().into()),
        unexpected => type_error_with_slice("(path: string)", unexpected),
    });
    result.add_fn("is_dir", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(path)] => Ok(Path::new(path.as_str()).is_dir().into()),
        unexpected => type_error_with_slice("(path: string)", unexpected),
    });
    result.add_fn("is_symlink", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(path)] => Ok(Path::new(path.as_str()).is_symlink().into()),
        unexpected => type_error_with_slice("(path: string)", unexpected),
    });
    result.add_fn("mkdirs", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(path)] => Ok(std::fs::create_dir_all(path.as_str())
            .map_err(|e| make_runtime_error!(format!("Failed to create directory {path}")))?
            .into()),
        unexpected => type_error_with_slice("(path: string)", unexpected),
    });
    result.add_fn("copy", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(src), Value::Str(dst)] => Ok(std::fs::copy(src.as_str(), dst.as_str())
            .map_err(|e| make_runtime_error!(format!("Failed to copy file {src} to {dst}: {e}")))?
            .into()),
        unexpected => type_error_with_slice("(src: string, dst: string)", unexpected),
    });
    result.add_fn("copy_dir", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(src), Value::Str(dst)] => Ok(copy_dir_internal(
            Path::new(src.as_str()),
            Path::new(dst.as_str()),
        )
        .map_err(|e| make_runtime_error!(format!("Failed to copy directory {src} to {dst}: {e}")))?
        .into()),
        unexpected => type_error_with_slice("(src: string, dst: string)", unexpected),
    });
    result.add_fn("glob", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(pattern)] => Ok(glob(pattern)?.into()),
        unexpected => type_error_with_slice("(pattern: string)", unexpected),
    });
    result.add_fn("copy_glob", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(pattern), Value::Str(dst)] => Ok(copy_glob(pattern, dst)?),
        unexpected => type_error_with_slice("(pattern: string, dst: string)", unexpected),
    });
    result.add_fn("temp_file", |ctx: &mut CallContext<'_>| match ctx.args() {
        [] => {
            let handle = tempfile::NamedTempFile::new().map_err(|e| {
                make_runtime_error!(format!("Failed to create named temp file: {}", e))
            })?;
            let path = handle.path().to_path_buf();
            Ok(koto::runtime::core_lib::io::File::system_file(handle, path))
        }
        unexpected => type_error_with_slice("()", unexpected),
    });
    result.add_fn("temp_dir", |ctx: &mut CallContext<'_>| match ctx.args() {
        [] => Ok(TempDir::new()?.into()),
        unexpected => type_error_with_slice("()", unexpected),
    });
    result.add_fn("absolute", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(path)] => Ok(absolute(path.as_str()).map_err(|e| make_runtime_error!(format!("Failed to absolutize path: {e}")))?.into()),
        unexpected => type_error_with_slice("(path: string)", unexpected),
    });
    result.add_fn("which", |ctx: &mut CallContext<'_>| match ctx.args() {
        [Value::Str(executable)] => Ok(which(executable.as_str()).map_err(|e| make_runtime_error!(format!("Failed to find executable path: {e}")))?.into()),
        unexpected => type_error_with_slice("(executable: string)", unexpected),
    });
    result
}

fn copy_dir_internal<P, Q>(src: P, dst: Q) -> std::io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_dir() {
            copy_dir_internal(&entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(&entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn glob(pattern: &str) -> Result<KList> {
    let glob = glob::glob(&pattern)
        .map_err(|e| make_runtime_error!(format!("Failed to evaluate glob pattern: {e}")))?;
    Ok(KList::with_data(
        glob.filter_map(|entry| entry.ok())
            .map(|entry| Value::Str(KString::from(entry.to_string_lossy().to_string())))
            .collect(),
    ))
}

pub fn copy_glob(pattern: &str, dst: &str) -> Result<Value> {
    let glob = glob::glob(pattern)
        .map_err(|e| make_runtime_error!(format!("Failed to evaluate glob pattern: {e}")))?;
    for entry in glob.filter_map(|entry| entry.ok()) {
        let ft = entry
            .symlink_metadata()
            .map(|m| m.file_type())
            .map_err(|e| make_runtime_error!("Failed to fetch file metadata: {e}"))?;
        if ft.is_dir() {
            copy_dir_internal(&entry, &Path::new(dst).join(&entry))
                .map_err(|e| make_runtime_error!(format!("Failed to copy directory: {e}")))?;
        } else {
            let to = Path::new(dst).join(&entry);
            let to_dir = to.parent();
            if let Some(to_dir) = to_dir {
                std::fs::create_dir_all(to_dir).map_err(|e| {
                    make_runtime_error!(format!(
                        "Failed to create directory {}: {e}",
                        to.to_string_lossy()
                    ))
                })?;
                std::fs::copy(&entry, to).map_err(|e| {
                    make_runtime_error!(format!(
                        "Failed to copy file {} to destination {dst}: {e}",
                        entry.to_string_lossy()
                    ))
                })?;
            }
        }
    }
    Ok(Value::Null)
}

pub fn absolute(path: &str) -> std::io::Result<String> {
    Path::new(path)
        .absolutize()
        .map(|p| p.to_str().unwrap().to_string())
}

pub fn which(executable: &str) -> std::io::Result<String> {
    which::which(executable)
        .map(|p| p.to_str().unwrap().to_string())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))
}
