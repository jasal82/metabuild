use path_absolutize::*;
use rune::runtime::VmError;
use rune::{Any, ContextError, Module};
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
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn glob(pattern: &str) -> Result<Vec<String>, VmError> {
    use path_slash::PathExt as _;
    let glob = glob::glob(pattern).map_err(|e| VmError::panic(e.to_string()))?;
    Ok(glob
        .filter_map(|entry| entry.ok())
        .map(|entry| (*entry.to_slash_lossy().into_owned()).into())
        .collect())
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

pub fn copy_glob(pattern: &str, dst: &str) -> Result<(), anyhow::Error> {
    let glob = glob::glob(pattern).map_err(|e| {
        VmError::panic(format!(
            "Failed to evaluate glob pattern: {}",
            e.to_string()
        ))
    })?;
    for entry in glob.filter_map(|entry| entry.ok()) {
        let ft = entry
            .symlink_metadata()
            .map(|m| m.file_type())
            .map_err(|e| {
                VmError::panic(format!("Failed to fetch file metadata: {}", e.to_string()))
            })?;
        if ft.is_dir() {
            copy_dir_internal(&entry, &Path::new(dst).join(&entry)).map_err(|e| {
                VmError::panic(format!("Failed to copy directory: {}", e.to_string()))
            })?;
        } else {
            let to = Path::new(dst).join(&entry);
            let to_dir = to.parent();
            if let Some(to_dir) = to_dir {
                std::fs::create_dir_all(&to_dir).map_err(|e| {
                    VmError::panic(format!(
                        "Failed to create directory {}: {}",
                        to.to_string_lossy(),
                        e.to_string()
                    ))
                })?;
                std::fs::copy(&entry, to).map_err(|e| {
                    VmError::panic(format!(
                        "Failed to copy file {} to destination {}: {}",
                        entry.to_string_lossy(),
                        dst,
                        e.to_string()
                    ))
                })?;
            }
        }
    }
    Ok(())
}

pub fn absolute(path: &str) -> std::io::Result<String> {
    Path::new(path)
        .absolutize()
        .map(|p| p.to_str().unwrap().to_string())
}

#[derive(Any)]
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

#[derive(Any)]
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

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("fs");
    module.function(["glob"], glob)?;
    module.function(["exists"], exists)?;
    module.function(["is_file"], is_file)?;
    module.function(["is_dir"], is_dir)?;
    module.function(["is_symlink"], is_symlink)?;
    module.function(["mkdirs"], mkdirs)?;
    module.function(["delete"], delete)?;
    module.function(["copy"], copy)?;
    module.function(["copy_dir"], copy_dir)?;
    module.function(["copy_glob"], copy_glob)?;
    module.function(["absolute"], absolute)?;
    module.ty::<TempDir>()?;
    module.function(["TempDir", "new"], TempDir::new)?;
    module.inst_fn("path", TempDir::path)?;
    module.ty::<File>()?;
    module.function(["File", "open"], File::open)?;
    module.function(["File", "create"], File::create)?;
    module.function(["File", "append"], File::append)?;
    module.function(["File", "temp"], File::temp)?;
    module.inst_fn("sync", File::sync)?;
    module.inst_fn("read", File::read)?;
    module.inst_fn("write", File::write)?;
    module.inst_fn("seek_start", File::seek_start)?;
    module.inst_fn("seek_end", File::seek_end)?;
    module.inst_fn("seek_current", File::seek_current)?;
    module.inst_fn("rewind", File::rewind)?;
    module.inst_fn("is_dir", File::is_dir)?;
    module.inst_fn("is_file", File::is_file)?;
    module.inst_fn("is_symlink", File::is_symlink)?;
    module.inst_fn("len", File::len)?;
    module.inst_fn("path", File::path)?;
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
