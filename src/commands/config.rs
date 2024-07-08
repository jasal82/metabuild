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
    pub index_url: Option<String>,
    #[serde(default)]
    pub artifactory_token: HashMap<String, String>,
}

pub enum ConfigScope {
    Global,
    Local,
}

enum ConfigKey {
    IndexUrl,
    ArtifactoryToken(String),
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
        .unwrap();
        let local = toml::from_str(
            std::fs::read_to_string(".mb/config.toml")
                .unwrap_or(String::new())
                .as_str(),
        )
        .unwrap();

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
            toml::to_string(&self.global)?,
        )?;

        std::fs::create_dir_all(".mb")?;
        std::fs::write(".mb/config.toml", toml::to_string(&self.local)?)?;
        Ok(())
    }

    fn parse_key(key: &str) -> Result<ConfigKey, Error> {
        if let Some(start) = key.find('[') {
            if key.ends_with(']') {
                let k = &key[..start];
                let i = &key[start+1..key.len()-1];
                return match k {
                    "artifactory_token" => Ok(ConfigKey::ArtifactoryToken(i.to_string())),
                    _ => Err(anyhow!("Invalid key '{k}'"))
                }
            }
        }

        match key {
            "index_url" => Ok(ConfigKey::IndexUrl),
            _ => Err(anyhow!("Invalid key '{key}'"))
        }
    }

    pub fn set(&mut self, key: &str, value: &str, scope: ConfigScope) -> Result<(), Error> {
        let target = match scope {
            ConfigScope::Global => &mut self.global,
            ConfigScope::Local => &mut self.local,
        };

        match Self::parse_key(key)? {
            ConfigKey::IndexUrl => {
                println!("Set config 'index_url'");
                target.index_url = Some(value.to_owned());
                self.write_and_update()
            },
            ConfigKey::ArtifactoryToken(url) => {
                println!("Set config 'artifactory_token.{url}'");
                target.artifactory_token.insert(url.to_string(), value.to_owned());
                self.write_and_update()
            }
        }
    }

    pub fn get(&mut self, key: &str) -> Result<&str, Error> {
        match Self::parse_key(key)? {
            ConfigKey::IndexUrl => self.merged.index_url.as_ref().map(String::as_str).ok_or(anyhow!("index_url is not set")),
            ConfigKey::ArtifactoryToken(url) => self.merged.artifactory_token.get(&url).map(String::as_str).ok_or(anyhow!("artifactory_token.{url} is not set")),
        }
    }

    pub fn remove(&mut self, key: &str, scope: ConfigScope) -> Result<(), Error> {
        let target = match scope {
            ConfigScope::Global => &mut self.global,
            ConfigScope::Local => &mut self.local,
        };

        match Self::parse_key(key)? {
            ConfigKey::IndexUrl => {
                if target.index_url.is_some() {
                    println!("Removed config 'index_url'");
                    target.index_url = None;
                    self.write_and_update()
                } else {
                    println!("Nothing to remove");
                    Ok(())
                }
            },
            ConfigKey::ArtifactoryToken(url) => {
                if target.artifactory_token.contains_key(&url) {
                    println!("Removed config 'artifactory_token.{url}'");
                    target.artifactory_token.remove(&url);
                    self.write_and_update()
                } else {
                    println!("Nothing to remove");
                    Ok(())
                }
            },
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
                    } else if let Some(hash) = reflect_value.downcast_ref::<HashMap<String, String>>() {
                        println!("{}:", field_name);
                        for (k, v) in hash {
                            let path = if k.contains(".") {
                                format!(r#"{field_name}."{k}""#)
                            } else {
                                format!("{field_name}.{k}")
                            };
                            let figment_value = self.figment.find_value(path.as_str()).expect("Failed to retrieve figment value");
                            let figment_source = self.figment.get_metadata(figment_value.tag())
                                .expect("Failed to retrieve figment metadata")
                                .source.as_ref().expect("Failed to retrieve figment source");
                            println!("  {}: {} ({})", k, v, figment_source);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn list(&self) -> Result<(), Error> {
        println!("index_url: URL of the package index; can be overridden in the project's manifest.toml; only SSH protocol is supported");
        println!("artifactory_token.[url]: Token to use for authentication at the Artifactory server with the specified URL");
        Ok(())
    }
}
