use rhai::{Engine, Module};
use std::path::Path;
use super::RhaiResult;

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

pub fn glob(pattern: &str) -> RhaiResult<rhai::Array> {
    use path_slash::PathExt as _;

    let mut result = rhai::Array::new();
    for entry in glob::glob(pattern).unwrap() {
        if let Ok(entry) = entry {
            result.push((*entry.to_slash_lossy()).into());
        }
    }
    Ok(result)
}

pub fn exists(path: &str) -> RhaiResult<bool> {
    Ok(Path::new(path).exists())
}

pub fn is_file(path: &str) -> RhaiResult<bool> {
    Ok(Path::new(path).is_file())
}

pub fn is_dir(path: &str) -> RhaiResult<bool> {
    Ok(Path::new(path).is_dir())
}

pub fn mkdirs(path: &str) -> RhaiResult<bool> {
    Ok(std::fs::create_dir_all(Path::new(path)).is_ok())
}

pub fn delete(path: &str) -> RhaiResult<bool> {
    let p = Path::new(path);
	if p.is_dir() {
		Ok(std::fs::remove_dir_all(p).is_ok())
	} else if p.is_file() {
		Ok(std::fs::remove_file(p).is_ok())
	} else {
		Ok(false)
	}
}

pub fn copy(src: &str, dst: &str) -> RhaiResult<bool> {
    Ok(std::fs::copy(src, dst).is_ok())
}

pub fn copy_dir(src: &str, dst: &str) -> RhaiResult<bool> {
    Ok(copy_dir_internal(src, dst).is_ok())
}

pub fn read_file(path: &str) -> RhaiResult<String> {
    Ok(std::fs::read_to_string(path).unwrap())
}

pub fn write_file(path: &str, content: &str) -> RhaiResult<bool> {
    Ok(std::fs::write(path, content).is_ok())
}

pub fn register(engine: &mut Engine) {
    let mut module = Module::new();
    module.set_native_fn("glob", glob);
    module.set_native_fn("exists", exists);
    module.set_native_fn("is_file", is_file);
    module.set_native_fn("is_dir", is_dir);
    module.set_native_fn("mkdirs", mkdirs);
    module.set_native_fn("delete", delete);
    module.set_native_fn("copy", copy);
    module.set_native_fn("copy_dir", copy_dir);
    module.set_native_fn("read_file", read_file);
    module.set_native_fn("write_file", write_file);
    engine.register_static_module("fs", module.into());
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_fs() {
        let temp_dir = tempdir().unwrap();

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
        assert!(read_file(temp_dir.path().join("dummy.txt").to_str().unwrap()).unwrap().contains("test"));
    }
}