mod util;
pub mod modem;

use crate::util::AsyncStream;
use crate::modem::connect_default as network_connect_default;

/// Completely setup docker connection to actually make http requests.
pub async fn connect_default() -> anyhow::Result<Box<dyn AsyncStream>> {
    let network_connection = network_connect_default().await?;
    Ok(network_connection)
}
