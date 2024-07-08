use anyhow::{Context, Error};
use koto::Koto;
use koto::{derive::*, prelude::*};
use std::fs;
use std::path::{Path, PathBuf};
use path_absolutize::Absolutize;

mod api;

#[derive(Clone, KotoCopy, KotoType)]
struct ModuleMetadata {
    path: PathBuf,
}

#[koto_impl]
impl ModuleMetadata {
    fn new(path: &Path) -> Self { Self { path: path.to_path_buf() } }

    #[koto_method]
    fn get_resource(ctx: MethodContext<Self>) -> koto::Result<KValue> {
        match ctx.args {
            [KValue::Str(resource_path)] => {
                let mod_path = &ctx.instance()?.path;
                Ok(mod_path.join(resource_path.as_str()).to_str().expect("Invalid path").into())
            },
            unexpected => type_error_with_slice("(resource_path: string)", unexpected),
        }
    }
}

impl KotoObject for ModuleMetadata {
    fn display(&self, ctx: &mut DisplayContext) -> koto::Result<()> {
        ctx.append("ModuleMetadata");
        Ok(())
    }
}

fn add_common_prelude(koto: &mut Koto) {
    let prelude = koto.prelude();
    prelude.insert("arch", api::arch::make_module());
    prelude.insert("cmd", api::cmd::make_module());
    prelude.insert("git", api::git::make_module());
    prelude.insert("http", api::http::make_module());
    prelude.insert("io_ext", api::io::make_module());
    prelude.insert("json", koto_json::make_module());
    prelude.insert("net", api::net::make_module());
    prelude.insert("regex", koto_regex::make_module());
    prelude.insert("sys", api::sys::make_module());
    prelude.insert("tempfile", koto_tempfile::make_module());
    prelude.insert("toml", koto_toml::make_module());
    prelude.insert("utils", api::utils::make_module());
    prelude.insert("yaml", koto_yaml::make_module());
}

fn load_dynamic_modules(koto: &mut Koto) -> Result<(), anyhow::Error> {
    // TODO replace with rust_search for better performance (https://github.com/ParthJadhav/rust_search)
    let mod_files = glob::glob("./.mb/deps/**/mod.koto")?;
    for mf in mod_files {
        let mf = mf?;
        let mod_dir = mf.parent().unwrap();
        let mod_name = mod_dir.file_name().unwrap().to_str().unwrap();

        let mut runtime = Koto::new();
        add_common_prelude(&mut runtime);
        runtime
            .set_script_path(Some(mf.clone()))
            .expect("Failed to set script path");
        let script: String = fs::read_to_string(&mf)?;
        runtime
            .compile_and_run(&script)
            .map_err(koto_error_to_anyhow)
            .context(format!("Error while compiling {}", mod_name))?;
        koto.prelude()
            .insert(koto::runtime::KString::from(mod_name), runtime.exports().clone());
    }

    Ok(())
}

fn inject_resources(koto: &mut Koto) -> Result<(), Error> {
    let module_map = KMap::new();
    let manifest_files = glob::glob("./.mb/deps/*/manifest.toml")?;
    for mf in manifest_files {
        let mf = mf?;
        let mod_dir = mf.parent().unwrap();
        let mod_name = mod_dir.file_name().unwrap().to_str().unwrap();
        let module_meta = ModuleMetadata::new(mod_dir.absolutize()?.as_ref());
        module_map.insert(mod_name.replace("-", "_").as_str(), KObject::from(module_meta));
    }

    let metabuild_map = KMap::new();
    metabuild_map.insert("modules", module_map);
    koto.prelude().insert("metabuild", metabuild_map);
    Ok(())
}

pub fn run_file(script_file: &Path) -> Result<(), anyhow::Error> {
    let mut koto = Koto::new();
    add_common_prelude(&mut koto);

    load_dynamic_modules(&mut koto).context("Error while loading dynamic modules")?;
    inject_resources(&mut koto).context("Error while injecting resources")?;

    koto.set_script_path(Some(script_file.to_path_buf()))
        .expect("Failed to set script path");
    let script = fs::read_to_string(script_file)?;

    koto.compile(&script)
        .map_err(koto_error_to_anyhow)
        .context("Error while compiling script")?;
    koto.run()
        .map_err(koto_error_to_anyhow)
        .context("Error while running script")?;
    Ok(())
}

fn koto_error_to_anyhow(e: koto::Error) -> anyhow::Error {
    // koto::Error doesn't implement Send, which anyhow requires, so render the error to a String
    Error::msg(e.to_string())
}
