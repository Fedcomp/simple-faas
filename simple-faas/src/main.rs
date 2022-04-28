mod config;

use env_logger::Env;
use log::{debug, info};
use self::config::Config;
use std::fs::File;
use warp::Filter;
use std::sync::Arc;
use simple_faas_docker::client::Client as DockerClient;
use simple_faas_docker::v1_37::Api as DockerApi;
use simple_faas_docker::v1_37::ContainerCreateArgs;
use warp::http::Response;

const DEFAULT_CONFIG_NAME: &str = "config.yml";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    info!("Starting");

    let config_reader = File::open(DEFAULT_CONFIG_NAME)?;
    let config: Config = serde_yaml::from_reader(config_reader)?;
    let config = Arc::new(config);
    let listen_host = config.listen_host;
    debug!("Loaded {} function(s) from config", config.functions.len());

    let config = warp::any().map(move || config.clone());

    let function_call_filter =
        warp::path!("functions" / String)
        .and(config)
        .and_then(function_call_handler);

    info!("Listening on {:?}", listen_host);

    warp::serve(function_call_filter)
        .run(listen_host)
        .await;

    Ok(())
}

async fn function_call_handler(name: String, config: Arc<Config>) -> Result<impl warp::Reply, warp::Rejection> {
    let response = match call_docker_function(name, config).await {
        Ok(output) => {
            Response::builder()
                .status(200)
                .body(output)
                .expect("Failed to construct a response")
        },
        Err(e) => {
            Response::builder()
                .status(500)
                .body(format!("Failed to call function: {}", e))
                .expect("Failed to construct a response")
        }
    };

    Ok(response)
}

async fn call_docker_function(name: String, config: Arc<Config>) -> anyhow::Result<String> {
    let client = DockerClient::new(config.docker_host.clone());
    let api = DockerApi::new(client);
    let container_create_opts = ContainerCreateArgs {
        Image: name.clone(),
        Cmd: None,
    };
    let container = api.containers().create(container_create_opts).await?;
    container.start().await?;
    container.wait().await?;
    let logs = container.logs().await?;
    container.delete().await?;

    Ok(logs)
}
