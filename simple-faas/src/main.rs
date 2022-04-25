use env_logger::Env;
use log::{debug, info};
use simple_faas::config::Config;
use simple_faas_docker::client::Client as DockerClient;
use simple_faas_docker::v1_37::Api as DockerApi;
use simple_faas_docker::v1_37::ContainerCreateArgs;
use std::fs::File;

const DEFAULT_CONFIG_NAME: &str = "config.yml";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    info!("Starting");

    let config_reader = File::open(DEFAULT_CONFIG_NAME)?;
    let config: Config = serde_yaml::from_reader(config_reader)?;
    debug!("Loaded {} function(s) from config", config.functions.len());

    let first_function = config.functions.iter().next().expect("No functions were loaded").1;

    let client = DockerClient::new(config.docker_host);
    let api = DockerApi::new(client);
    let container_create_opts = ContainerCreateArgs {
        Image: first_function.image.clone(),
        Cmd: None,
    };
    let container = api.containers().create(container_create_opts).await?;
    container.start().await?;
    container.wait().await?;
    container.delete().await?;
    Ok(())
}
