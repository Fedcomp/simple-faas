use env_logger::Env;
use log::{debug, info};
use simple_faas::config::Config;
use std::fs::File;

const DEFAULT_CONFIG_NAME: &str = "config.yml";

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    info!("Starting");

    let config_reader = File::open(DEFAULT_CONFIG_NAME)?;
    let config: Config = serde_yaml::from_reader(config_reader)?;
    debug!("Loaded {} function(s) from config", config.functions.len());

    Ok(())
}
