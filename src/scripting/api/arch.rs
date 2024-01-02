use flate2::{read::GzDecoder, write::GzEncoder};
use koto::prelude::*;
use koto::runtime::Result;
use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;
use tar::{Archive, Builder};
use zip::ZipWriter;

#[derive(Clone)]
struct TarGz {
    inner: Rc<RefCell<Option<Builder<GzEncoder<File>>>>>,
    needs_finish: bool,
}

impl TarGz {
    pub fn create(file: &str) -> Result<Self> {
        let f = File::create(file)
            .map_err(|e| make_runtime_error!(format!("Failed to create file: {e}")))?;
        let enc: GzEncoder<File> = GzEncoder::new(f, flate2::Compression::default());
        Ok(Self {
            inner: Rc::new(Some(Builder::new(enc)).into()),
            needs_finish: true,
        })
    }

    pub fn append_file(&mut self, arch_path: &str, file: &str) -> Result<Value> {
        let mut f: File = File::open(file)
            .map_err(|e| make_runtime_error!(format!("Failed to open file: {e}")))?;
        self.inner
            .borrow_mut()
            .as_mut()
            .unwrap()
            .append_file(arch_path, &mut f)
            .map_err(|e| make_runtime_error!(format!("Failed to append file: {e}")))?;
        Ok(Value::Null)
    }

    pub fn append_dir_all(&mut self, arch_path: &str, path: &str) -> Result<Value> {
        self.inner
            .borrow_mut()
            .as_mut()
            .unwrap()
            .append_dir_all(arch_path, path)
            .map_err(|e| make_runtime_error!(format!("Failed to append dir: {e}")))?;
        Ok(Value::Null)
    }

    pub fn finish(&mut self) -> Result<Value> {
        self.inner
            .borrow_mut()
            .take()
            .unwrap()
            .into_inner()
            .map_err(|e| make_runtime_error!(format!("Failed to finish tar.gz: {e}")))?;
        self.needs_finish = false;
        Ok(Value::Null)
    }
}

impl Drop for TarGz {
    fn drop(&mut self) {
        if self.needs_finish {
            self.inner
                .borrow_mut()
                .take()
                .unwrap()
                .into_inner()
                .unwrap();
        }
    }
}

impl KotoType for TarGz {
    const TYPE: &'static str = "TarGz";
}

impl KotoObject for TarGz {
    fn object_type(&self) -> KString {
        KString::from("TarGz")
    }

    fn copy(&self) -> KObject {
        self.clone().into()
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        TARGZ_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("TarGz");
        Ok(())
    }
}

impl From<TarGz> for Value {
    fn from(tar_gz: TarGz) -> Self {
        KObject::from(tar_gz).into()
    }
}

fn make_targz_entries() -> ValueMap {
    ObjectEntryBuilder::<TarGz>::new()
        .method("append_file", |ctx| match ctx.args {
            [Value::Str(arch_path), Value::Str(file)] => {
                ctx.instance_mut()?.append_file(arch_path, file)
            }
            unexpected => type_error_with_slice("(arch_path: string, file: string)", unexpected),
        })
        .method("append_dir_all", |ctx| match ctx.args {
            [Value::Str(arch_path), Value::Str(path)] => {
                ctx.instance_mut()?.append_dir_all(arch_path, path)
            }
            unexpected => type_error_with_slice("(arch_path: string, path: string)", unexpected),
        })
        .method("finish", |ctx| match ctx.args {
            [] => ctx.instance_mut()?.finish(),
            unexpected => type_error_with_slice("()", unexpected),
        })
        .build()
}

thread_local! {
    static TARGZ_TYPE_STRING: KString = TarGz::TYPE.into();
    static TARGZ_ENTRIES: ValueMap = make_targz_entries();
}

#[derive(Clone)]
struct Zip {
    inner: Rc<RefCell<ZipWriter<File>>>,
    needs_finish: bool,
}

impl Zip {
    pub fn create(file: &str) -> Result<Self> {
        let f = File::create(file)
            .map_err(|e| make_runtime_error!(format!("Failed to create file: {e}")))?;
        Ok(Self {
            inner: Rc::new(ZipWriter::new(f).into()),
            needs_finish: true,
        })
    }

    pub fn append_file(&mut self, arch_path: &str, file: &str) -> Result<Value> {
        let mut f = File::open(file)
            .map_err(|e| make_runtime_error!(format!("Failed to open file: {e}")))?;
        self.inner
            .borrow_mut()
            .start_file(arch_path, Default::default())
            .map_err(|e| make_runtime_error!(format!("Failed to add file: {e}")))?;
        std::io::copy(&mut f, &mut *self.inner.borrow_mut())
            .map_err(|e| make_runtime_error!(format!("Failed to add file: {e}")))?;
        Ok(Value::Null)
    }

    pub fn append_dir_all(&mut self, arch_path: &str, path: &str) -> Result<Value> {
        for entry in std::fs::read_dir(path)
            .map_err(|e| make_runtime_error!(format!("Failed to read dir: {e}")))?
        {
            let entry =
                entry.map_err(|e| make_runtime_error!(format!("Failed to read dir entry: {e}")))?;
            let path = entry.path();
            let arch_path = match arch_path {
                "." | "" => path.file_name().unwrap().to_str().unwrap().to_string(),
                _ => format!(
                    "{}/{}",
                    arch_path,
                    path.file_name().unwrap().to_str().unwrap()
                ),
            };
            if path.is_dir() {
                self.append_dir_all(&arch_path, path.to_str().unwrap())?;
            } else {
                self.append_file(&arch_path, path.to_str().unwrap())?;
            }
        }
        Ok(Value::Null)
    }

    pub fn finish(&mut self) -> Result<Value> {
        self.inner
            .borrow_mut()
            .finish()
            .map_err(|e| make_runtime_error!(format!("Failed to finish zip: {e}")))?;
        self.needs_finish = false;
        Ok(Value::Null)
    }
}

impl Drop for Zip {
    fn drop(&mut self) {
        if self.needs_finish {
            self.inner.borrow_mut().finish().unwrap();
        }
    }
}

impl KotoType for Zip {
    const TYPE: &'static str = "Zip";
}

impl KotoObject for Zip {
    fn object_type(&self) -> KString {
        KString::from("Zip")
    }

    fn copy(&self) -> KObject {
        self.clone().into()
    }

    fn lookup(&self, key: &ValueKey) -> Option<Value> {
        ZIP_ENTRIES.with(|entries| entries.get(key).cloned())
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append("Zip");
        Ok(())
    }
}

impl From<Zip> for Value {
    fn from(zip: Zip) -> Self {
        KObject::from(zip).into()
    }
}

fn make_zip_entries() -> ValueMap {
    ObjectEntryBuilder::<Zip>::new()
        .method("append_file", |ctx| match ctx.args {
            [Value::Str(arch_path), Value::Str(file)] => {
                ctx.instance_mut()?.append_file(arch_path, file)
            }
            unexpected => type_error_with_slice("(arch_path: string, file: string)", unexpected),
        })
        .method("append_dir_all", |ctx| match ctx.args {
            [Value::Str(arch_path), Value::Str(path)] => {
                ctx.instance_mut()?.append_dir_all(arch_path, path)
            }
            unexpected => type_error_with_slice("(arch_path: string, path: string)", unexpected),
        })
        .method("finish", |ctx| match ctx.args {
            [] => ctx.instance_mut()?.finish(),
            unexpected => type_error_with_slice("()", unexpected),
        })
        .build()
}

thread_local! {
    static ZIP_TYPE_STRING: KString = Zip::TYPE.into();
    static ZIP_ENTRIES: ValueMap = make_zip_entries();
}

pub fn extract(file: &str, dst: &str) -> Result<Value> {
    let ext = std::path::Path::new(file)
        .extension()
        .unwrap()
        .to_str()
        .unwrap();
    match ext {
        "zip" => {
            let zip = File::open(file)
                .map_err(|e| make_runtime_error!(format!("Failed to open file: {e}")))?;
            let mut archive = zip::ZipArchive::new(zip)
                .map_err(|e| make_runtime_error!(format!("Failed to open zip archive: {e}")))?;
            archive
                .extract(dst)
                .map_err(|e| make_runtime_error!(format!("Failed to extract zip archive: {e}")))?;
            Ok(Value::Null)
        }
        "gz" | "tgz" => {
            let tar_gz = File::open(file)
                .map_err(|e| make_runtime_error!(format!("Failed to open file: {e}")))?;
            let tar = GzDecoder::new(tar_gz);
            let mut archive = Archive::new(tar);
            archive
                .unpack(dst)
                .map_err(|e| make_runtime_error!(format!("Failed to extract tar archive: {e}")))?;
            Ok(Value::Null)
        }
        _ => {
            return Err(make_runtime_error!(format!(
                "Unsupported archive type: {}",
                ext
            )))
        }
    }
}

pub fn make_module() -> KMap {
    let result = KMap::with_type("arch");
    result.add_fn("zip", |ctx| match ctx.args() {
        [Value::Str(file)] => Zip::create(file).map(|zip| zip.into()),
        unexpected => type_error_with_slice("a filename", unexpected),
    });
    result.add_fn("targz", |ctx| match ctx.args() {
        [Value::Str(file)] => TarGz::create(file).map(|targz| targz.into()),
        unexpected => type_error_with_slice("a filename", unexpected),
    });
    result.add_fn("extract", |ctx| match ctx.args() {
        [Value::Str(file), Value::Str(dst)] => extract(file, dst),
        unexpected => type_error_with_slice("(file: string, dst: string)", unexpected),
    });

    result
}
