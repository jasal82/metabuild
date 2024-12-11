use anyhow::{anyhow, Error};
use bevy_reflect::{Reflect, Struct, TypeInfo};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Config {
    pub figment: Figment,
    pub merged: ConfigData,
    pub global: ConfigData,
    pub local: ConfigData,
}

#[derive(Serialize, Deserialize, Reflect, Default)]
pub struct ConfigData {
    pub index: Option<String>,
    pub artifactory_token: HashMap<String, String>,
}

pub enum ConfigScope {
    Global,
    Local,
}

impl Config {
    pub fn new() -> Self {
        let figment = Figment::from(Serialized::defaults(ConfigData::default()))
            .merge(Toml::file(
                dirs::home_dir().unwrap().join(".mb/config.toml"),
            ))
            .merge(Toml::file(".mb/config.toml"))
            .merge(Env::prefixed("METABUILD_"));

        let merged = figment.extract::<ConfigData>().unwrap();

        let global = toml::from_str(
            std::fs::read_to_string(dirs::home_dir().unwrap().join(".mb/config.toml"))
                .unwrap_or(String::new())
                .as_str(),
        )
        .unwrap_or_default();
        let local = toml::from_str(
            std::fs::read_to_string(".mb/config.toml")
                .unwrap_or(String::new())
                .as_str(),
        )
        .unwrap_or_default();

        Config {
            figment,
            merged,
            global,
            local,
        }
    }

    fn write_and_update(&mut self) -> Result<(), Error> {
        std::fs::create_dir_all(dirs::home_dir().unwrap().join(".mb"))?;
        std::fs::write(
            dirs::home_dir().unwrap().join(".mb/config.toml"),
            toml::to_string_pretty(&self.global)?,
        )?;

        std::fs::create_dir_all(".mb")?;
        std::fs::write(".mb/config.toml", toml::to_string_pretty(&self.local)?)?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str, scope: ConfigScope) -> Result<(), Error> {
        let target = match scope {
            ConfigScope::Global => &mut self.global,
            ConfigScope::Local => &mut self.local,
        };

        match key {
            "index" => {
                println!("Set config 'index'");
                target.index = Some(value.to_owned());
                self.write_and_update()
            },
            _ => {
                println!("Invalid config key");
                Err(anyhow!("Invalid config key"))
            },
        }
    }

    pub fn set_token(&mut self, url: &str, token: &str, scope: ConfigScope) -> Result<(), Error> {
        let target = match scope {
            ConfigScope::Global => &mut self.global,
            ConfigScope::Local => &mut self.local,
        };

        println!("Set artifactory token for url '{}'", url);
        target.artifactory_token.insert(url.to_owned(), token.to_owned());
        self.write_and_update()
    }

    pub fn get(&mut self, key: &str) -> Result<&str, Error> {
        match key {
            "index" => {
                self.merged.index.as_ref().map(String::as_str).ok_or(anyhow!("Config 'index' is not set"))
            },
            _ => {
                println!("Invalid config key");
                Err(anyhow!("Invalid config key"))
            }
        }
    }

    pub fn get_token(&mut self, url: &str) -> Result<&str, Error> {
        self.merged
            .artifactory_token
            .get(url)
            .map(String::as_str)
            .ok_or(anyhow!("Artifactory token for url '{}' is not set", url))
    }

    pub fn remove(&mut self, key: &str, scope: ConfigScope) -> Result<(), Error> {
        let target = match scope {
            ConfigScope::Global => &mut self.global,
            ConfigScope::Local => &mut self.local,
        };

        match key {
            "index" => {
                if target.index.is_some() {
                    println!("Removed config 'index'");
                    target.index = None;
                    self.write_and_update()
                } else {
                    println!("Nothing to remove");
                    Ok(())
                }
            },
            _ => {
                println!("Invalid config key");
                Err(anyhow!("Invalid config key"))
            }
        }
    }

    pub fn remove_token(&mut self, url: &str, scope: ConfigScope) -> Result<(), Error> {
        let target = match scope {
            ConfigScope::Global => &mut self.global,
            ConfigScope::Local => &mut self.local,
        };

        if target.artifactory_token.remove(url).is_some() {
            println!("Removed artifactory token for url '{}'", url);
            self.write_and_update()
        } else {
            println!("Nothing to remove");
            Ok(())
        }
    }

    pub fn show(&mut self) -> Result<(), Error> {
        if let Some(type_info) = self.merged.get_represented_type_info() {
            if let TypeInfo::Struct(struct_info) = type_info {
                for (i, reflect_value) in self.merged.iter_fields().enumerate() {
                    let field_name = struct_info.field_at(i).expect("Failed to retrieve field reflection info").name();
                    if let Some(option) = reflect_value.downcast_ref::<Option<String>>() {
                        if let Some(value) = option.as_ref() {
                            let figment_value = self.figment.find_value(field_name).expect("Failed to retrieve figment value");
                            let figment_source = self.figment.get_metadata(figment_value.tag())
                                .expect("Failed to retrieve figment metadata")
                                .source.as_ref().expect("Failed to retrieve figment source");
                            println!(
                                "{}: {} ({})",
                                field_name,
                                value,
                                figment_source
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn list(&self) -> Result<(), Error> {
        println!("index: URL of the package index; can be overridden in the project's manifest.toml; only SSH protocol is supported");
        Ok(())
    }
}
