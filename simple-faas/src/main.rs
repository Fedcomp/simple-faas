use env_logger::Env;
use log::{debug, info};
use simple_faas::config::Config;
use simple_faas_docker::modem::connect_default;
use std::fs::File;

const DEFAULT_CONFIG_NAME: &str = "config.yml";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    info!("Starting");

    let config_reader = File::open(DEFAULT_CONFIG_NAME)?;
    let config: Config = serde_yaml::from_reader(config_reader)?;
    debug!("Loaded {} function(s) from config", config.functions.len());

    debug!("Connecting to docker");
    let _connection = connect_default().await?;
    info!("Connected to docker");
    Ok(())
}
