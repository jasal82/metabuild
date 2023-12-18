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
    use path_slash::PathExt as _;
    let glob = glob::glob(&pattern)
        .map_err(|e| make_runtime_error!(format!("Failed to evaluate glob pattern: {e}")))?;
    Ok(KList::with_data(
        glob.filter_map(|entry| entry.ok())
            .map(|entry| Value::Str(KString::from(entry.to_slash_lossy().to_string())))
            .collect(),
    ))
}

pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub fn is_symlink(path: &str) -> bool {
    Path::new(path)
        .symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

pub fn mkdirs(path: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(Path::new(path))
}

pub fn delete(path: &str) -> std::io::Result<()> {
    let p = Path::new(path);
    if p.is_dir() {
        std::fs::remove_dir_all(p)
    } else {
        std::fs::remove_file(p)
    }
}

pub fn copy(src: &str, dst: &str) -> std::io::Result<u64> {
    std::fs::copy(src, dst)
}

pub fn copy_dir(src: &str, dst: &str) -> std::io::Result<()> {
    copy_dir_internal(Path::new(src), Path::new(dst))
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

/*
#[derive(Clone, Debug)]
struct TempDir {
    dir: tempfile::TempDir,
}

impl TempDir {
    fn new() -> std::io::Result<Self> {
        Ok(Self {
            dir: tempfile::tempdir()?,
        })
    }

    fn path(&self) -> String {
        self.dir.path().to_str().unwrap().to_string()
    }
}

enum FileKind {
    TempFile(tempfile::NamedTempFile),
    RegularFile { file: std::fs::File, path: String },
}

#[derive(Clone, Debug)]
struct File {
    inner: FileKind,
}

impl File {
    fn open(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            inner: FileKind::RegularFile {
                file: std::fs::File::open(path)?,
                path: path.to_string(),
            },
        })
    }

    fn create(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            inner: FileKind::RegularFile {
                file: std::fs::File::create(path)?,
                path: path.to_string(),
            },
        })
    }

    fn append(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            inner: FileKind::RegularFile {
                file: std::fs::OpenOptions::new().append(true).open(path)?,
                path: path.to_string(),
            },
        })
    }

    fn temp() -> std::io::Result<Self> {
        Ok(Self {
            inner: FileKind::TempFile(tempfile::NamedTempFile::new()?),
        })
    }

    fn sync(&self) -> std::io::Result<()> {
        match self.inner {
            FileKind::TempFile(ref f) => f.as_file().sync_all(),
            FileKind::RegularFile { ref file, .. } => file.sync_all(),
        }
    }

    fn read(&mut self) -> std::io::Result<String> {
        let mut s = String::new();
        match self.inner {
            FileKind::TempFile(ref mut f) => f.read_to_string(&mut s)?,
            FileKind::RegularFile { ref mut file, .. } => file.read_to_string(&mut s)?,
        };
        Ok(s)
    }

    fn write(&mut self, test: &str) -> std::io::Result<()> {
        match self.inner {
            FileKind::TempFile(ref mut f) => f.write_all(test.as_bytes()),
            FileKind::RegularFile { ref mut file, .. } => file.write_all(test.as_bytes()),
        }
    }

    fn seek_start(&mut self, pos: u64) -> std::io::Result<u64> {
        match self.inner {
            FileKind::TempFile(ref mut f) => f.seek(std::io::SeekFrom::Start(pos)),
            FileKind::RegularFile { ref mut file, .. } => file.seek(std::io::SeekFrom::Start(pos)),
        }
    }

    fn seek_end(&mut self, pos: i64) -> std::io::Result<u64> {
        match self.inner {
            FileKind::TempFile(ref mut f) => f.seek(std::io::SeekFrom::End(pos)),
            FileKind::RegularFile { ref mut file, .. } => file.seek(std::io::SeekFrom::End(pos)),
        }
    }

    fn seek_current(&mut self, pos: i64) -> std::io::Result<u64> {
        match self.inner {
            FileKind::TempFile(ref mut f) => f.seek(std::io::SeekFrom::Current(pos)),
            FileKind::RegularFile { ref mut file, .. } => {
                file.seek(std::io::SeekFrom::Current(pos))
            }
        }
    }

    fn rewind(&mut self) -> std::io::Result<()> {
        match self.inner {
            FileKind::TempFile(ref mut f) => f.rewind(),
            FileKind::RegularFile { ref mut file, .. } => file.rewind(),
        }
    }

    fn is_dir(&self) -> bool {
        match self.inner {
            FileKind::TempFile(ref f) => f
                .as_file()
                .metadata()
                .expect("Cannot fetch file metadata")
                .is_dir(),
            FileKind::RegularFile { ref file, .. } => file
                .metadata()
                .expect("Cannot fetch file metadata")
                .is_dir(),
        }
    }

    fn is_file(&self) -> bool {
        match self.inner {
            FileKind::TempFile(ref f) => f
                .as_file()
                .metadata()
                .expect("Cannot fetch file metadata")
                .is_file(),
            FileKind::RegularFile { ref file, .. } => file
                .metadata()
                .expect("Cannot fetch file metadata")
                .is_file(),
        }
    }

    fn is_symlink(&self) -> bool {
        match self.inner {
            FileKind::TempFile(ref f) => f
                .as_file()
                .metadata()
                .expect("Cannot fetch file metadata")
                .file_type()
                .is_symlink(),
            FileKind::RegularFile { ref file, .. } => file
                .metadata()
                .expect("Cannot fetch file metadata")
                .file_type()
                .is_symlink(),
        }
    }

    fn len(&self) -> u64 {
        match self.inner {
            FileKind::TempFile(ref f) => f
                .as_file()
                .metadata()
                .expect("Cannot fetch file metadata")
                .len(),
            FileKind::RegularFile { ref file, .. } => {
                file.metadata().expect("Cannot fetch file metadata").len()
            }
        }
    }

    fn path(&self) -> String {
        match self.inner {
            FileKind::TempFile(ref f) => f.path().to_str().unwrap().to_string(),
            FileKind::RegularFile { ref path, .. } => path.to_string(),
        }
    }
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("fs");
    result.add_fn("glob", {
        |ctx| match ctx.args() {
            [Value::Str(pattern)]
        }
    })
    let mut module = Module::with_crate("fs")?;

    module.function_meta(glob)?;
    module.function_meta(exists)?;
    module.function_meta(is_file)?;
    module.function_meta(is_dir)?;
    module.function_meta(is_symlink)?;
    module.function_meta(mkdirs)?;
    module.function_meta(delete)?;
    module.function_meta(copy)?;
    module.function_meta(copy_dir)?;
    module.function_meta(copy_glob)?;
    module.function_meta(absolute)?;
    module.function_meta(which)?;

    module.ty::<TempDir>()?;
    module.function_meta(TempDir::new)?;
    module.function_meta(TempDir::path)?;

    module.ty::<File>()?;
    module.function_meta(File::open)?;
    module.function_meta(File::create)?;
    module.function_meta(File::append)?;
    module.function_meta(File::temp)?;
    module.function_meta(File::sync)?;
    module.function_meta(File::read)?;
    module.function_meta(File::write)?;
    module.function_meta(File::seek_start)?;
    module.function_meta(File::seek_end)?;
    module.function_meta(File::seek_current)?;
    module.function_meta(File::rewind)?;
    module.function_meta(File::is_dir)?;
    module.function_meta(File::is_file)?;
    module.function_meta(File::is_symlink)?;
    module.function_meta(File::len)?;
    module.function_meta(File::path)?;

    Ok(module)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_fs() {
        /*let temp_dir = tempdir().unwrap();

        let files = glob("tests/**/
        *.txt").unwrap();
        assert!(files.iter().any(|e| e.clone().into_immutable_string().unwrap() == "tests/subdir/dummy.txt"));
        assert!(exists("tests/subdir/dummy.txt").unwrap());
        assert!(!exists("tests/subdir/doesnt_exist").unwrap());
        assert!(is_file("tests/subdir/dummy.txt").unwrap());
        assert!(!is_file("tests/subdir").unwrap());
        assert!(!is_file("tests/subdir/doesnt_exist").unwrap());
        assert!(is_dir("tests/subdir").unwrap());
        assert!(!is_dir("tests/subdir/dummy.txt").unwrap());
        assert!(!is_dir("tests/subdir/doesnt_exist").unwrap());
        assert!(mkdirs(temp_dir.path().join("a/b/c").to_str().unwrap()).unwrap());
        assert!(exists(temp_dir.path().join("a/b/c").to_str().unwrap()).unwrap());
        assert!(delete(temp_dir.path().join("a/b/c").to_str().unwrap()).unwrap());
        assert!(!exists(temp_dir.path().join("a/b/c").to_str().unwrap()).unwrap());
        assert!(copy("tests/subdir/dummy.txt", temp_dir.path().join("dummy.txt").to_str().unwrap()).unwrap());
        assert!(exists(temp_dir.path().join("dummy.txt").to_str().unwrap()).unwrap());
        assert!(copy_dir("tests/subdir", temp_dir.path().join("subdir").to_str().unwrap()).unwrap());
        assert!(exists(temp_dir.path().join("subdir/dummy.txt").to_str().unwrap()).unwrap());
        assert!(read_file(temp_dir.path().join("dummy.txt").to_str().unwrap()).unwrap().contains("no content"));
        assert!(write_file(temp_dir.path().join("dummy.txt").to_str().unwrap(), "test").unwrap());
        assert!(read_file(temp_dir.path().join("dummy.txt").to_str().unwrap()).unwrap().contains("test"));*/
    }
}
*/
