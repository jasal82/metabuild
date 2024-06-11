use anyhow::{Context, Error};
use koto::Koto;
use std::fs;
use std::path::{Path, PathBuf};

mod api;

struct DynamicModule {
    name: String,
    file: PathBuf,
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

fn load_dynamic_modules(koto: &mut Koto, modules: &[DynamicModule]) -> Result<(), anyhow::Error> {
    for module in modules {
        println!("Loading module {}", module.name);
        let mut runtime = Koto::new();
        add_common_prelude(&mut runtime);
        runtime
            .set_script_path(Some(module.file.clone()))
            .expect("Failed to set script path");
        let script: String = fs::read_to_string(&module.file)?;
        runtime
            .compile_and_run(&script)
            .map_err(koto_error_to_anyhow)
            .context(format!("Error while compiling {}", module.name))?;
        koto.prelude()
            .insert(koto::runtime::KString::from(module.name.as_str()), runtime.exports().clone());
    }

    Ok(())
}

pub fn run_file(script_file: &Path) -> Result<(), anyhow::Error> {
    let mut koto = Koto::new();
    add_common_prelude(&mut koto);

    let mod_files = glob::glob("./.mb/modules/**/mod.koto")?;
    let mut modules = vec![];
    for mf in mod_files {
        let mf = mf?;
        let mod_dir = mf.parent().unwrap();
        let mod_name = mod_dir.file_name().unwrap().to_str().unwrap();
        modules.push(DynamicModule {
            name: mod_name.to_string(),
            file: mf,
        });
    }

    load_dynamic_modules(&mut koto, &modules).context("Error while loading dynamic modules")?;

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
