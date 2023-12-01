use path_absolutize::*;
use rune::runtime::{VmError, VmResult};
use rune::{Any, ContextError, Module, vm_try};
use std::io::{Read, Seek, Write};
use std::path::Path;

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

#[rune::function]
pub fn glob(pattern: &str) -> VmResult<Vec<String>> {
    use path_slash::PathExt as _;
    let glob = vm_try!(glob::glob(&pattern).map_err(|e| VmError::panic(e.to_string())));
    VmResult::Ok(glob
        .filter_map(|entry| entry.ok())
        .map(|entry| (*entry.to_slash_lossy().into_owned()).into())
        .collect())
}

#[rune::function]
pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

#[rune::function]
pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

#[rune::function]
pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

#[rune::function]
pub fn is_symlink(path: &str) -> bool {
    Path::new(path)
        .symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

#[rune::function]
pub fn mkdirs(path: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(Path::new(path))
}

#[rune::function]
pub fn delete(path: &str) -> std::io::Result<()> {
    let p = Path::new(path);
    if p.is_dir() {
        std::fs::remove_dir_all(p)
    } else {
        std::fs::remove_file(p)
    }
}

#[rune::function]
pub fn copy(src: &str, dst: &str) -> std::io::Result<u64> {
    std::fs::copy(src, dst)
}

#[rune::function]
pub fn copy_dir(src: &str, dst: &str) -> std::io::Result<()> {
    copy_dir_internal(Path::new(src), Path::new(dst))
}

#[rune::function]
pub fn copy_glob(pattern: &str, dst: &str) -> Result<(), anyhow::Error> {
    let glob = glob::glob(pattern)
        .map_err(|e| VmError::panic(format!("Failed to evaluate glob pattern: {e}")))?;
    for entry in glob.filter_map(|entry| entry.ok()) {
        let ft = entry
            .symlink_metadata()
            .map(|m| m.file_type())
            .map_err(|e| VmError::panic(format!("Failed to fetch file metadata: {e}")))?;
        if ft.is_dir() {
            copy_dir_internal(&entry, &Path::new(dst).join(&entry))
                .map_err(|e| VmError::panic(format!("Failed to copy directory: {e}")))?;
        } else {
            let to = Path::new(dst).join(&entry);
            let to_dir = to.parent();
            if let Some(to_dir) = to_dir {
                std::fs::create_dir_all(to_dir).map_err(|e| {
                    VmError::panic(format!(
                        "Failed to create directory {}: {e}",
                        to.to_string_lossy()
                    ))
                })?;
                std::fs::copy(&entry, to).map_err(|e| {
                    VmError::panic(format!(
                        "Failed to copy file {} to destination {dst}: {e}",
                        entry.to_string_lossy()
                    ))
                })?;
            }
        }
    }
    Ok(())
}

#[rune::function]
pub fn absolute(path: &str) -> std::io::Result<String> {
    Path::new(path)
        .absolutize()
        .map(|p| p.to_str().unwrap().to_string())
}

#[rune::function]
pub fn which(executable: &str) -> std::io::Result<String> {
    which::which(executable)
        .map(|p| p.to_str().unwrap().to_string())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))
}

#[derive(Any)]
#[rune(item = ::fs)]
struct TempDir {
    dir: tempfile::TempDir,
}

impl TempDir {
    #[rune::function(path = Self::new)]
    fn new() -> std::io::Result<Self> {
        Ok(Self {
            dir: tempfile::tempdir()?,
        })
    }

    #[rune::function]
    fn path(&self) -> String {
        self.dir.path().to_str().unwrap().to_string()
    }
}

enum FileKind {
    TempFile(tempfile::NamedTempFile),
    RegularFile { file: std::fs::File, path: String },
}

#[derive(Any)]
#[rune(item = ::fs)]
struct File {
    inner: FileKind,
}

impl File {
    #[rune::function(path = Self::open)]
    fn open(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            inner: FileKind::RegularFile {
                file: std::fs::File::open(path)?,
                path: path.to_string(),
            },
        })
    }

    #[rune::function(path = Self::create)]
    fn create(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            inner: FileKind::RegularFile {
                file: std::fs::File::create(path)?,
                path: path.to_string(),
            },
        })
    }

    #[rune::function(path = Self::append)]
    fn append(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            inner: FileKind::RegularFile {
                file: std::fs::OpenOptions::new().append(true).open(path)?,
                path: path.to_string(),
            },
        })
    }

    #[rune::function(path = Self::temp)]
    fn temp() -> std::io::Result<Self> {
        Ok(Self {
            inner: FileKind::TempFile(tempfile::NamedTempFile::new()?),
        })
    }

    #[rune::function]
    fn sync(&self) -> std::io::Result<()> {
        match self.inner {
            FileKind::TempFile(ref f) => f.as_file().sync_all(),
            FileKind::RegularFile { ref file, .. } => file.sync_all(),
        }
    }

    #[rune::function]
    fn read(&mut self) -> std::io::Result<String> {
        let mut s = String::new();
        match self.inner {
            FileKind::TempFile(ref mut f) => f.read_to_string(&mut s)?,
            FileKind::RegularFile { ref mut file, .. } => file.read_to_string(&mut s)?,
        };
        Ok(s)
    }

    #[rune::function]
    fn write(&mut self, test: &str) -> std::io::Result<()> {
        match self.inner {
            FileKind::TempFile(ref mut f) => f.write_all(test.as_bytes()),
            FileKind::RegularFile { ref mut file, .. } => file.write_all(test.as_bytes()),
        }
    }

    #[rune::function]
    fn seek_start(&mut self, pos: u64) -> std::io::Result<u64> {
        match self.inner {
            FileKind::TempFile(ref mut f) => f.seek(std::io::SeekFrom::Start(pos)),
            FileKind::RegularFile { ref mut file, .. } => file.seek(std::io::SeekFrom::Start(pos)),
        }
    }

    #[rune::function]
    fn seek_end(&mut self, pos: i64) -> std::io::Result<u64> {
        match self.inner {
            FileKind::TempFile(ref mut f) => f.seek(std::io::SeekFrom::End(pos)),
            FileKind::RegularFile { ref mut file, .. } => file.seek(std::io::SeekFrom::End(pos)),
        }
    }

    #[rune::function]
    fn seek_current(&mut self, pos: i64) -> std::io::Result<u64> {
        match self.inner {
            FileKind::TempFile(ref mut f) => f.seek(std::io::SeekFrom::Current(pos)),
            FileKind::RegularFile { ref mut file, .. } => {
                file.seek(std::io::SeekFrom::Current(pos))
            }
        }
    }

    #[rune::function]
    fn rewind(&mut self) -> std::io::Result<()> {
        match self.inner {
            FileKind::TempFile(ref mut f) => f.rewind(),
            FileKind::RegularFile { ref mut file, .. } => file.rewind(),
        }
    }

    #[rune::function]
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

    #[rune::function]
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

    #[rune::function]
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

    #[rune::function]
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

    #[rune::function]
    fn path(&self) -> String {
        match self.inner {
            FileKind::TempFile(ref f) => f.path().to_str().unwrap().to_string(),
            FileKind::RegularFile { ref path, .. } => path.to_string(),
        }
    }
}

pub fn module() -> Result<Module, ContextError> {
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
