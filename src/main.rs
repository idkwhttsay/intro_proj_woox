mod ws_types;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{Message, Result};
use tokio::sync::mpsc;
use serde_json::json;
use crate::ws_types::{WsOrderbookUpdate, WsOrderbookUpdateData};

const WEBSOCKET_URL: &str = "wss://wss.woox.io/v3/public";
const DEPTH: usize = 50;
const TICKER: &str = "PERP_ETH_USDT";


#[tokio::main]
async fn main() -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<WsOrderbookUpdateData>(100);

    let (ws_stream, _) = connect_async(WEBSOCKET_URL).await?;
    println!("WebSocket connected to {}", WEBSOCKET_URL);

    let (mut write, mut read) = ws_stream.split();

    let tx_ws_read = tx.clone();

    // spawn a task to handle incoming WebSocket messages
    tokio::spawn(async move {
        let subscribe_message = serde_json::to_string(&json!({
            "cmd": "SUBSCRIBE",
            "params": [format!("orderbookupdate@{}@{}", TICKER, DEPTH)],
        })).unwrap();

        // send the initial subscribe message
        write.send(Message::Text(subscribe_message)).await.unwrap();
        
        // read and print the initial connection response
        if let Some(Ok(Message::Text(text))) = read.next().await {
            println!("Received initial message: {}", text);
        }

        // read the stream of messages from WebSocket
        while let Some(message) = read.next().await {
            if let Ok(Message::Text(text)) = message {
                match serde_json::from_str::<WsOrderbookUpdate>(&text) {
                    Ok(update) => {
                        if update.topic.starts_with("orderbookupdate") {
                            if let Err(e) = tx_ws_read.send(update.data).await {
                                eprintln!("Failed to send update via channel: {}", e);
                            }
                        }
                    }

                    Err(e) => {
                        eprintln!("Failed to deserialize message: {}, error: {}", text, e);
                        break;
                    }
                }
            }
        }
    });

    while let Some(update) = rx.recv().await {
        println!("{:#?}", update);
    }

    Ok(())
}

