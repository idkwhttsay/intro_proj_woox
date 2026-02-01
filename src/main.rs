mod dto;
mod orderbook;
mod ws_client;

use crate::ws_client::connect_ws;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    connect_ws().await?;
    Ok(())
}

