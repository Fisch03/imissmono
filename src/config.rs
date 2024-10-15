use config::Config;
use serde::Deserialize;
use std::sync::OnceLock;

use holodex::model::*;

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub server: ServerConfig,
    pub holodex: HolodexConfig,
}

pub struct ConfigRoot {
    pub server: ServerConfig,
    pub holodex: Holodex,
}

#[derive(Debug, Deserialize)]
pub struct HolodexConfig {
    api_key: String,
    channel_id: String,
}

pub struct Holodex {
    pub client: holodex::Client,
    pub channel: holodex::model::id::ChannelId,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

pub fn config() -> &'static ConfigRoot {
    static CONFIG: OnceLock<ConfigRoot> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let config = Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name("config/local").required(false))
            .build()
            .expect("Failed to build config");

        let config: RawConfig = config
            .try_deserialize()
            .expect("Failed to deserialize config");

        ConfigRoot {
            server: config.server,
            holodex: Holodex::from_config(config.holodex),
        }
    })
}

impl Holodex {
    fn from_config(config: HolodexConfig) -> Self {
        let client = holodex::Client::new(&config.api_key).unwrap();
        let channel: holodex::model::id::ChannelId = config
            .channel_id
            .parse()
            .expect("Failed to parse channel ID");

        Self { client, channel }
    }

    pub fn videos(&self) -> Result<Vec<Video>, holodex::errors::Error> {
        let videos = self.channel.videos(&self.client)?;

        Ok(videos.into_items())
    }
}
