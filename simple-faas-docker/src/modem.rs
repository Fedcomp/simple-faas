//! Docker remote API network layer module

use crate::util::AsyncStream;
use tokio::net::TcpStream;

/// Use docker-cli logic to setup only network connection stream
/// without any data being sent.
/// 
/// For now just connects to addr i need in my own project.
/// TODO: Actually parse ENV variables and connect to docker socket by default.
pub async fn connect_default() -> anyhow::Result<Box<dyn AsyncStream>> {
    // Temporary path for my work project
    const TMP_HACKY_PATH: &str = "docker:2376";

    let stream = TcpStream::connect(TMP_HACKY_PATH).await?;
    Ok(Box::new(stream))
}
