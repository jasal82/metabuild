use flate2::{read::GzDecoder, write::GzEncoder};
use rune::{Any, ContextError, Module};
use std::fs::File;
use std::path::Path;
use tar::{Archive, Builder};
use zip::ZipWriter;

#[derive(Any)]
#[rune(item = ::arch)]
struct TarGz {
    inner: Builder<GzEncoder<File>>,
}

impl TarGz {
    #[rune::function(path = Self::create)]
    pub fn create(file: &str) -> Result<Self, anyhow::Error> {
        let f = File::create(file)?;
        let enc = GzEncoder::new(f, flate2::Compression::default());
        Ok(Self {
            inner: Builder::new(enc),
        })
    }

    #[rune::function]
    pub fn append_file(&mut self, arch_path: &str, file: &str) -> Result<(), anyhow::Error> {
        let mut f = File::open(file)?;
        self.inner.append_file(arch_path, &mut f).map_err(|e| e.into())
    }

    #[rune::function]
    pub fn append_dir_all(&mut self, arch_path: &str, path: &str) -> Result<(), anyhow::Error> {
        self.inner.append_dir_all(arch_path, path).map_err(|e| e.into())
    }
}

#[derive(Any)]
#[rune(item = ::arch)]
struct Zip {
    inner: ZipWriter<File>
}

impl Zip {
    #[rune::function(path = Self::create)]
    pub fn create(file: &str) -> Result<Self, anyhow::Error> {
        let f = File::create(file)?;
        Ok(Self {
            inner: ZipWriter::new(f),
        })
    }

    #[rune::function]
    pub fn append_file(&mut self, arch_path: &str, file: &str) -> Result<(), anyhow::Error> {
        let mut f = File::open(file)?;
        self.inner.start_file(arch_path, Default::default())?;
        std::io::copy(&mut f, &mut self.inner)?;
        Ok(())
    }

    #[rune::function]
    pub fn append_dir_all(&mut self, arch_path: &str, path: &str) -> Result<(), anyhow::Error> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let arch_path = match arch_path {
                "." | "" => path.file_name().unwrap().to_str().unwrap().to_string(),
                _ => format!("{}/{}", arch_path, path.file_name().unwrap().to_str().unwrap()),
            };
            if path.is_dir() {
                self.__rune_fn__append_dir_all(&arch_path, path.to_str().unwrap())?;
            } else {
                self.__rune_fn__append_file(&arch_path, path.to_str().unwrap())?;
            }
        }
        Ok(())
    }
}

#[rune::function]
pub fn extract(file: &str, dst: &str) -> std::io::Result<()> {
    let tar_gz = File::open(file)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(dst)?;
    Ok(())
}

#[rune::function]
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
    let mut module = Module::with_crate("arch")?;
    module.ty::<TarGz>()?;
    module.function_meta(TarGz::create)?;
    module.function_meta(TarGz::append_file)?;
    module.function_meta(TarGz::append_dir_all)?;

    module.ty::<Zip>()?;
    module.function_meta(Zip::create)?;
    module.function_meta(Zip::append_file)?;
    module.function_meta(Zip::append_dir_all)?;

    module.function_meta(extract)?;
    module.function_meta(create)?;
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_extract() {
        let temp_dir = tempdir().unwrap();
        __rune_fn__extract(
            "tests/test.tar.gz",
            temp_dir.path().as_os_str().to_str().unwrap(),
        )
        .unwrap();
        assert!(temp_dir.path().join("arch.rs").exists());
    }

    #[test]
    fn test_create() {
        let temp_file = NamedTempFile::new().unwrap();
        __rune_fn__create(
            temp_file.path().as_os_str().to_str().unwrap(),
            "src/scripting/api",
        )
        .unwrap();
        assert!(temp_file.path().exists());
        assert!(temp_file.path().metadata().unwrap().len() > 0);
    }
}
