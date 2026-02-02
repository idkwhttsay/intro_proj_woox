mod dto;
mod orderbook;
mod ws_client;
mod orderbook_service;
mod consts;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    orderbook_service::run_orderbook_service().await;
    Ok(())
}

