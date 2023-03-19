use flate2::{read::GzDecoder, write::GzEncoder};
use rune::{ContextError, Module};
use std::fs::File;
use std::path::Path;
use tar::Archive;

pub fn extract(file: &str, dst: &str) -> std::io::Result<()> {
    let tar_gz = File::open(file)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(dst)?;
    Ok(())
}

pub fn create(file: &str, dir: &str) -> std::io::Result<()> {
    let tar_gz = File::create(file)?;
    let enc = GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);
    let path = Path::new(dir);
    if path.is_dir() {
        tar.append_dir_all("", path)?;
    }
    Ok(())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("arch");
    module.function(["extract"], extract)?;
    module.function(["create"], create)?;
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_extract() {
        let temp_dir = tempdir().unwrap();
        extract("tests/test.tar.gz", temp_dir.path().as_os_str().to_str().unwrap()).unwrap();
        assert!(temp_dir.path().join("arch.rs").exists());
    }

    #[test]
    fn test_create() {
        let temp_file = NamedTempFile::new().unwrap();
        create(temp_file.path().as_os_str().to_str().unwrap(), "src/scripting/api").unwrap();
        assert!(temp_file.path().exists());
        assert!(temp_file.path().metadata().unwrap().len() > 0);
    }
}