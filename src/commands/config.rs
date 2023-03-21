use anyhow::anyhow;
use figment::{providers::{Env, Format, Serialized, Toml}, Figment};
use serde::{Deserialize, Serialize};

pub struct Config {
    pub figment: Figment,
    pub merged: ConfigData,
    pub global: ConfigData,
    pub local: ConfigData,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigData {
    pub gitlab_token: Option<String>,
}

pub enum ConfigScope {
    Global,
    Local,
}

impl Default for ConfigData {
    fn default() -> Self {
        ConfigData {
            gitlab_token: None,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        let figment = Figment::from(Serialized::defaults(ConfigData::default()))
            .merge(Toml::file(
                dirs::home_dir()
                    .unwrap()
                    .join(".mb/config.toml")
            ))
            .merge(Toml::file(".mb/config.toml"))
            .merge(Env::prefixed("METABUILD_"));

        let merged = figment.extract::<ConfigData>().unwrap();

        let global = toml::from_str(std::fs::read_to_string(
            dirs::home_dir().unwrap().join(".mb/config.toml")
        ).unwrap_or(String::new()).as_str()).unwrap();
        let local = toml::from_str(std::fs::read_to_string(".mb/config.toml").unwrap_or(String::new()).as_str()).unwrap();
        
        Config { figment, merged, global, local }
    }

    fn write_and_update(&mut self) -> Result<(), anyhow::Error> {
        std::fs::create_dir_all(dirs::home_dir().unwrap().join(".mb"))?;
        std::fs::write(
            dirs::home_dir().unwrap().join(".mb/config.toml"),
            toml::to_string(&self.global)?
        )?;

        std::fs::create_dir_all(".mb")?;
        std::fs::write(".mb/config.toml", toml::to_string(&self.local)?)?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str, scope: ConfigScope) -> Result<(), anyhow::Error> {
        if key == "gitlab_token" {
            match scope {
                ConfigScope::Global => {
                    self.global.gitlab_token = Some(value.to_owned());
                    self.write_and_update()
                },
                ConfigScope::Local => {
                    self.local.gitlab_token = Some(value.to_owned());
                    self.write_and_update()
                }
            }
        } else {
            Err(anyhow!("Invalid key"))
        }
    }

    pub fn get(&mut self, key: &str) -> Result<(), anyhow::Error> {
        if key == "gitlab_token" {
            println!("{}", self.merged.gitlab_token.as_ref().unwrap());
            Ok(())
        } else {
            Err(anyhow!("Invalid key"))
        }
    }

    pub fn list(&mut self) -> Result<(), anyhow::Error> {
        if self.merged.gitlab_token.is_some() {
            let name = self.figment.find_value("gitlab_token").unwrap();
            println!("gitlab_token: {} ({})", self.merged.gitlab_token.as_ref().unwrap(), self.figment.get_metadata(name.tag()).unwrap().source.as_ref().unwrap());
        }
        Ok(())
    }
}