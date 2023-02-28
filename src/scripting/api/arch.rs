use flate2::{read::GzDecoder, write::GzEncoder};
use rhai::{Engine, Module};
use std::fs::File;
use std::path::Path;
use super::RhaiResult;
use tar::Archive;

pub fn extract(file: &str, dst: &str) -> RhaiResult<()> {
    let tar_gz = File::open(file).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(dst).unwrap();
    Ok(())
}

pub fn create(file: &str, dir: &str) -> RhaiResult<()> {
    let tar_gz = File::create(file).unwrap();
    let enc = GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);
    let path = Path::new(dir);
    if path.is_dir() {
        tar.append_dir_all("", path).unwrap();
    }
    Ok(())
}

pub fn register(engine: &mut Engine) {
    let mut module = Module::new();
    module.set_native_fn("extract", extract);
    module.set_native_fn("create", create);
    engine.register_static_module("arch", module.into());
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_extract() {
        let temp_dir = tempdir().unwrap();
        extract("resources/test.tar.gz", temp_dir.path().as_os_str().to_str().unwrap()).unwrap();
        assert!(temp_dir.path().join("arch.rs").exists());
    }

    #[test]
    fn test_create() {
        let temp_file = NamedTempFile::new().unwrap();
        create(temp_file.path().as_os_str().to_str().unwrap(), "src/api").unwrap();
        assert!(temp_file.path().exists());
        assert!(temp_file.path().metadata().unwrap().len() > 0);
    }
}