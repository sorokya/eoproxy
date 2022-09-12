use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: String,
}

#[derive(Debug, Deserialize)]
pub struct Proxy {
    pub host: String,
    pub port: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub proxy: Proxy,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("Config.toml"))
            .add_source(File::with_name("Config.local.toml").required(false))
            .build()?;

        s.try_deserialize()
    }
}