use std::{fs::File, path::PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub pk: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub db_user: Option<String>,
    pub db_password: Option<String>,
    pub db_name: String,
    pub base_dir: PathBuf,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // read from  ./app.yml, or /etc/config/app.yml, or from env CHAT_CONFIG
        let ret = match (
            File::open("./analytics.yml"),
            File::open("/etc/config/analytics.yml"),
            std::env::var("ANALYTICS_CONFIG"),
        ) {
            (Ok(f), _, _) => serde_yaml::from_reader(f),
            (_, Ok(f), _) => serde_yaml::from_reader(f),
            (_, _, Ok(f)) => serde_yaml::from_reader(File::open(f)?),
            _ => bail!("Config file not found"),
        };
        Ok(ret?)
    }
}
