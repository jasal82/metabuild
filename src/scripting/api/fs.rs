use rune::{ContextError, Module};
use rune::runtime::VmError;
use std::path::Path;

fn copy_dir_internal(src: &str, dst: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_dir() {
            copy_dir_internal(
                &entry.path().into_os_string().into_string().unwrap(),
                Path::new(dst).join(entry.file_name()).to_str().unwrap())?;
        } else {
            std::fs::copy(entry.path(), 
                Path::new(dst).join(entry.file_name()).to_str().unwrap())?;
        }
    }
    Ok(())
}

pub fn glob(pattern: &str) -> Result<Vec<String>, VmError> {
    use path_slash::PathExt as _;
    let glob = glob::glob(pattern).map_err(|e| VmError::panic(e.to_string()))?;
    Ok(glob.filter_map(|entry| entry.ok()).map(|entry| (*entry.to_slash_lossy().into_owned()).into()).collect())
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
    copy_dir_internal(src, dst)
}

pub fn read_file(path: &str) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

pub fn write_file(path: &str, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("arch");
    module.function(["glob"], glob)?;
    module.function(["exists"], exists)?;
    module.function(["is_file"], is_file)?;
    module.function(["is_dir"], is_dir)?;
    module.function(["mkdirs"], mkdirs)?;
    module.function(["delete"], delete)?;
    module.function(["copy"], copy)?;
    module.function(["copy_dir"], copy_dir)?;
    module.function(["read_file"], read_file)?;
    module.function(["write_file"], write_file)?;
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_fs() {
        /*let temp_dir = tempdir().unwrap();

        let files = glob("tests/**/*.txt").unwrap();
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