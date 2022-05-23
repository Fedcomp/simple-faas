mod config;

use self::config::Config;
use env_logger::Env;
use log::{debug, info};
use simple_faas_docker::client::Client as DockerClient;
use simple_faas_docker::v1_37::Api as DockerApi;
use simple_faas_docker::v1_37::ContainerCreateArgs;
use std::sync::Arc;
use warp::http::Response;
use warp::Filter;
use warp::reject;
use bytes::Bytes;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("simple_faas=trace")).init();
    info!("Starting");

    let config = config::read_default()?;
    let config = Arc::new(config);
    let listen_host = config.listen_host;
    debug!("Loaded {} function(s) from config", config.functions.len());
    info!(
        "Loaded auth for next docker registries: {}",
        config
            .docker_config
            .auths
            .iter()
            .map(|(host, _auth)| host)
            .fold(String::new(), |a, b| a + ", " + b)
    );

    info!("Pulling function images");
    for (function_name, function) in config.functions.iter() {
        debug!(
            "Pulling function {} from {}",
            function_name,
            function.image.clone()
        );

        pull_image(function.image.clone(), config.clone()).await?;
    }
    info!("Successfuly pulled all images");

    let config = warp::any().map(move || config.clone());

    let function_call_filter = warp::path!("functions" / String)
        .and(warp::body::content_length_limit(1024 * 1024))
        .and(warp::body::bytes())
        .and(config)
        .and_then(function_call_handler);

    info!("Listening on {:?}", listen_host);

    warp::serve(function_call_filter).run(listen_host).await;

    Ok(())
}

async fn pull_image(tag: String, config: Arc<Config>) -> anyhow::Result<()> {
    let api = docker_api(config);
    api.images().pull(tag).await?;

    Ok(())
}

async fn function_call_handler(
    name: String,
    body: Bytes, 
    config: Arc<Config>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let iter_name = name.clone();
    let function = config
        .functions
        .iter()
        .find(move |(f_name, _f_data)| *f_name == &iter_name)
        .map(|(_name, data)| data);

    debug!("API INPUT CHECK");
    dbg!(&body);
    let input = match body.is_empty() {
        true => None,
        false => Some(body),
    };
    dbg!(&input);

    let function = match function {
        Some(f) => f,
        None => return Err(reject())
    };

    let response = match call_docker_function(function.image.clone(), input, config).await {
        Ok(output) => Response::builder()
            .status(200)
            .body(output)
            .expect("Failed to construct a response"),
        Err(e) => Response::builder()
            .status(500)
            .body(format!("Failed to call function: {}", e))
            .expect("Failed to construct a response"),
    };

    Ok(response)
}

async fn call_docker_function(name: String, input: Option<Bytes>, config: Arc<Config>) -> anyhow::Result<String> {
    let api = docker_api(config);
    let container_create_opts = ContainerCreateArgs {
        Image: name.clone(),
        Cmd: None,
        AttachStdin: true,
        OpenStdin: true,
        Tty: true,
    };
    let container = api.containers().create(container_create_opts).await?;
    container.start().await?;

    if let Some(input) = input {
        container.send_to_stdin(input).await?;
    }

    container.wait().await?;
    let function_logs = container.logs().await?;
    container.delete().await?;

    dbg!(&function_logs);

    Ok(function_logs)
}

fn docker_api(config: Arc<Config>) -> DockerApi {
    let client = DockerClient::new(config.docker_host.clone());
    let api = DockerApi::new(client, config.docker_config.clone());
    api
}
