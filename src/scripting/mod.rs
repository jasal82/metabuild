use anyhow::{Context, Error};
use koto::Koto;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

mod api;

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
    let ns_tools = koto::runtime::KMap::new();
    let manifest_files = glob::glob("./.mb/deps/*/manifest.toml")?;
    for mf in manifest_files {
        let mf = mf?;
        let mod_dir = mf.parent().unwrap();
        let mod_name = mod_dir.file_name().unwrap().to_str().unwrap();
        let manifest_content = fs::read_to_string(&mf)?;
        let toml = manifest_content.parse::<toml::Table>().unwrap();
        if let Some(tools) = toml.get("tools").and_then(toml::Value::as_table) {
            let ns_mod_tools = koto::runtime::KMap::new();
            for (k, v) in tools.iter() {
                if let Some(metadata) = v.as_table() {
                    if let Some(path_entry) = metadata.get("path") {
                        if let Some(path) = path_entry.as_str() {
                            ns_mod_tools.insert(k.as_str(), mod_dir.join(path).to_str().unwrap());
                        }
                    }
                }
            }
            ns_tools.insert(mod_name.replace("-", "_").as_str(), ns_mod_tools);
        }
    }

    let ns_metabuild = koto::runtime::KMap::new();
    ns_metabuild.insert("tools", ns_tools);
    koto.prelude().insert("metabuild", ns_metabuild);
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
