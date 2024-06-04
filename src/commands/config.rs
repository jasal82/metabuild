use anyhow::anyhow;
use bevy_reflect::{Reflect, Struct, TypeInfo};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

pub struct Config {
    pub figment: Figment,
    pub merged: ConfigData,
    pub global: ConfigData,
    pub local: ConfigData,
}

#[derive(Serialize, Deserialize, Reflect, Default)]
pub struct ConfigData {
    pub gitlab_token: Option<String>,
    pub index_url: Option<String>,
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

    fn write_and_update(&mut self) -> Result<(), anyhow::Error> {
        std::fs::create_dir_all(dirs::home_dir().unwrap().join(".mb"))?;
        std::fs::write(
            dirs::home_dir().unwrap().join(".mb/config.toml"),
            toml::to_string(&self.global)?,
        )?;

        std::fs::create_dir_all(".mb")?;
        std::fs::write(".mb/config.toml", toml::to_string(&self.local)?)?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str, scope: ConfigScope) -> Result<(), anyhow::Error> {
        let target = match scope {
            ConfigScope::Global => &mut self.global,
            ConfigScope::Local => &mut self.local,
        };

        match key {
            "gitlab_token" => {
                println!("Set config 'gitlab_token'");
                target.gitlab_token = Some(value.to_owned());
                self.write_and_update()
            },
            "index_url" => {
                println!("Set config 'index_url'");
                target.index_url = Some(value.to_owned());
                self.write_and_update()
            },
            _ => Err(anyhow!("Invalid key"))
        }
    }

    pub fn get(&mut self, key: &str) -> Result<&str, anyhow::Error> {
        match key {
            "gitlab_token" => self.merged.gitlab_token.as_ref().map(String::as_str).ok_or(anyhow!("gitlab_token is not set")),
            "index_url" => self.merged.index_url.as_ref().map(String::as_str).ok_or(anyhow!("index_url is not set")),
            _ => Err(anyhow!("Invalid key"))
        }
    }

    pub fn remove(&mut self, key: &str, scope: ConfigScope) -> Result<(), anyhow::Error> {
        let target = match scope {
            ConfigScope::Global => &mut self.global,
            ConfigScope::Local => &mut self.local,
        };

        match key {
            "gitlab_token" => {
                if target.gitlab_token.is_some() {
                    println!("Removed config 'gitlab_token'");
                    target.gitlab_token = None;
                    self.write_and_update()
                } else {
                    println!("Nothing to remove");
                    Ok(())
                }
            },
            "index_url" => {
                if target.index_url.is_some() {
                    println!("Removed config 'index_url'");
                    target.index_url = None;
                    self.write_and_update()
                } else {
                    println!("Nothing to remove");
                    Ok(())
                }
            },
            _ => {
                println!("Invalid key");
                Ok(())
            }
        }
    }

    pub fn list(&mut self) -> Result<(), anyhow::Error> {
        if let Some(type_info) = self.merged.get_represented_type_info() {
            if let TypeInfo::Struct(struct_info) = type_info {
                for (i, reflect_value) in self.merged.iter_fields().enumerate() {
                    let field_name = struct_info.field_at(i).expect("Failed to retrieve field reflection info").name();
                    let figment_value = self.figment.find_value(field_name).expect("Failed to retrieve figment value");
                    let figment_source = self.figment.get_metadata(figment_value.tag())
                        .expect("Failed to retrieve figment metadata")
                        .source.as_ref().expect("Failed to retrieve figment source");
                    if let Some(option) = reflect_value.downcast_ref::<Option<String>>() {
                        if let Some(value) = option.as_ref() {
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
}
