use flate2::{read::GzDecoder, write::GzEncoder};
use koto::{derive::*, prelude::*, PtrMut, Result};
use std::fs::File;
use std::path::Path;
use tar::{Archive, Builder};
use zip::ZipWriter;

#[derive(Clone, KotoCopy, KotoType)]
struct TarGz {
    inner: PtrMut<Option<Builder<GzEncoder<File>>>>,
    needs_finish: bool,
}

#[koto_impl]
impl TarGz {
    pub fn create(file: KString) -> Result<Self> {
        let f = File::create(file.as_str())
            .map_err(|e| koto::runtime::Error::from(format!("Failed to create file: {e}")))?;
        let enc: GzEncoder<File> = GzEncoder::new(f, flate2::Compression::default());
        Ok(Self {
            inner: Some(Builder::new(enc)).into(),
            needs_finish: true,
        })
    }

    #[koto_method]
    pub fn append_file(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(arch_path), KValue::Str(file)] => {
                let mut f: File = File::open(file.as_str())
                    .map_err(|e| koto::runtime::Error::from(format!("Failed to open file: {e}")))?;
                ctx.instance_mut()?
                    .inner
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .append_file(arch_path.as_str(), &mut f)
                    .map_err(|e| koto::runtime::Error::from(format!("Failed to append file: {e}")))?;
                Ok(KValue::Null)
            }
            unexpected => type_error_with_slice("(arch_path: string, file: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn append_dir_all(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(arch_path), KValue::Str(path)] => {
                ctx.instance_mut()?
                    .inner
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .append_dir_all(arch_path.as_str(), path.as_str())
                    .map_err(|e| koto::runtime::Error::from(format!("Failed to append dir: {e}"))).unwrap();
                Ok(KValue::Null)
            }
            unexpected => type_error_with_slice("(arch_path: string, path: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn finish(&mut self) -> Result<KValue> {
        self.inner
            .borrow_mut()
            .take()
            .unwrap()
            .into_inner().map_err(|e| koto::runtime::Error::from(format!("Failed to finish tar.gz: {e}")))?
            .finish().map_err(|e| koto::runtime::Error::from(format!("Failed to finish tar.gz: {e}")))?;
        self.needs_finish = false;
        Ok(KValue::Null)
    }
}

impl Drop for TarGz {
    fn drop(&mut self) {
        if self.needs_finish {
            self.inner
                .borrow_mut()
                .take()
                .unwrap()
                .into_inner().map_err(|e| koto::runtime::Error::from(format!("Failed to finish tar.gz: {e}"))).unwrap()
                .finish().map_err(|e| koto::runtime::Error::from(format!("Failed to finish tar.gz: {e}"))).unwrap();
        }
    }
}

impl KotoObject for TarGz {
    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("TarGz");
        Ok(())
    }
}

impl From<TarGz> for KValue {
    fn from(tar_gz: TarGz) -> Self {
        KObject::from(tar_gz).into()
    }
}

#[derive(Clone, KotoCopy, KotoType)]
struct Zip {
    inner: PtrMut<ZipWriter<File>>,
    needs_finish: bool,
}

#[koto_impl]
impl Zip {
    pub fn create(file: KString) -> Result<Self> {
        let f = File::create(file.as_str())
            .map_err(|e| koto::runtime::Error::from(format!("Failed to create file: {e}")))?;
        Ok(Self {
            inner: ZipWriter::new(f).into(),
            needs_finish: true,
        })
    }

    fn append_file_internal(&mut self, arch_path: &str, file: &Path) -> std::result::Result<(), anyhow::Error> {
        let mut f = File::open(file)?;
        self.inner.borrow_mut().start_file(arch_path, Default::default())?;
        std::io::copy(&mut f, &mut *self.inner.borrow_mut())?;
        Ok(())
    }

    fn append_dir_all_internal(&mut self, arch_path: &str, path: &Path) -> std::result::Result<(), anyhow::Error> {
        for entry in std::fs::read_dir(path)?
        {
            let path = entry?.path();
            let arch_path = match arch_path {
                "." | "" => path.file_name().unwrap().to_str().unwrap().to_string(),
                _ => format!(
                    "{}/{}",
                    arch_path,
                    path.file_name().unwrap().to_str().unwrap()
                ),
            };
            if path.is_dir() {
                self.append_dir_all_internal(&arch_path, path.as_path())?;
            } else {
                self.append_file_internal(&arch_path, path.as_path())?;
            }
        }

        Ok(())
    }

    #[koto_method]
    pub fn append_file(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(arch_path), KValue::Str(file)] => {
                ctx.instance_mut()?.append_file_internal(arch_path.as_str(), Path::new(file.as_str()))
                    .map(|_| KValue::Null)
                    .map_err(|_| koto::runtime::Error::from("Failed to append file"))
            }
            unexpected => type_error_with_slice("(arch_path: string, file: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn append_dir_all(ctx: MethodContext<Self>) -> Result<KValue> {
        match ctx.args {
            [KValue::Str(arch_path), KValue::Str(path)] => {
                ctx.instance_mut()?.append_dir_all_internal(arch_path.as_str(), Path::new(path.as_str()))
                    .map(|_| KValue::Null)
                    .map_err(|_| koto::runtime::Error::from("Failed to append dir"))
            }
            unexpected => type_error_with_slice("(arch_path: string, path: string)", unexpected),
        }
    }

    #[koto_method]
    pub fn finish(&mut self) -> Result<KValue> {
        self.inner
            .borrow_mut()
            .finish()
            .map_err(|e| koto::runtime::Error::from(format!("Failed to finish zip: {e}")))?;
        self.needs_finish = false;
        Ok(KValue::Null)
    }
}

impl Drop for Zip {
    fn drop(&mut self) {
        if self.needs_finish {
            self.inner.borrow_mut().finish().unwrap();
        }
    }
}

impl KotoObject for Zip {
    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Zip");
        Ok(())
    }
}

impl From<Zip> for KValue {
    fn from(zip: Zip) -> Self {
        KObject::from(zip).into()
    }
}

pub fn extract(file: &str, dst: &str) -> Result<KValue> {
    let ext = Path::new(file)
        .extension()
        .unwrap()
        .to_str()
        .unwrap();
    match ext {
        "zip" => {
            let zip = File::open(file)
                .map_err(|e| koto::runtime::Error::from(format!("Failed to open file: {e}")))?;
            let mut archive = zip::ZipArchive::new(zip)
                .map_err(|e| koto::runtime::Error::from(format!("Failed to open zip archive: {e}")))?;
            archive
                .extract(dst)
                .map_err(|e| koto::runtime::Error::from(format!("Failed to extract zip archive: {e}")))?;
            Ok(KValue::Null)
        }
        "gz" | "tgz" => {
            let tar_gz = File::open(file)
                .map_err(|e| koto::runtime::Error::from(format!("Failed to open file: {e}")))?;
            let tar = GzDecoder::new(tar_gz);
            let mut archive = Archive::new(tar);
            archive
                .unpack(dst)
                .map_err(|e| koto::runtime::Error::from(format!("Failed to extract tar archive: {e}")))?;
            Ok(KValue::Null)
        }
        _ => {
            runtime_error!(format!(
                "Unsupported archive type: {}",
                ext
            ))
        }
    }
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("arch");
    result.add_fn("zip", |ctx| match ctx.args() {
        [KValue::Str(file)] => Zip::create(file.clone()).map(|zip| zip.into()),
        unexpected => type_error_with_slice("a filename", unexpected),
    });
    result.add_fn("targz", |ctx| match ctx.args() {
        [KValue::Str(file)] => TarGz::create(file.clone()).map(|targz| targz.into()),
        unexpected => type_error_with_slice("a filename", unexpected),
    });
    result.add_fn("extract", |ctx| match ctx.args() {
        [KValue::Str(file), KValue::Str(dst)] => extract(file, dst),
        unexpected => type_error_with_slice("(file: string, dst: string)", unexpected),
    });

    result
}
