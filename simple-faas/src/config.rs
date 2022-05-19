use anyhow::Context;
use serde::{Deserialize, Serialize};
use simple_faas_docker::auth::{self, DockerConfig};
use std::collections::HashMap;
use std::fs::File;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const DEFAULT_CONFIG_NAME: &str = "config.yml";

pub fn read_default() -> anyhow::Result<Config> {
    let config_reader =
        File::open(DEFAULT_CONFIG_NAME).with_context(|| "Failed to open app config")?;
    let mut config: Config = serde_yaml::from_reader(config_reader)?;
    let docker_config = auth::read_default()?;
    config.docker_config = docker_config;

    Ok(config)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub version: u64,
    pub docker_host: String,
    #[serde(default = "default_listen_host")]
    pub listen_host: SocketAddr,
    pub functions: HashMap<String, FunctionData>,
    #[serde(default)]
    pub docker_config: DockerConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionData {
    pub image: String,
}

fn default_listen_host() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
}
