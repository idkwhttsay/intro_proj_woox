use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{WebSocketStream, connect_async, MaybeTlsStream};
use tokio_tungstenite::tungstenite::{Message, Result};
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use serde_json::json;
use crate::dto::{WsOrderbookUpdate, WsOrderbookUpdateData};
use crate::consts::{WEBSOCKET_URL, TICKER, DEPTH, CHANNEL_BUFFER_SIZE};

async fn handle_ws_stream(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    tx: mpsc::Sender<WsOrderbookUpdateData>,
) {
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
                    if update.topic.starts_with("orderbookupdate")
                        && let Err(e) = tx.send(update.data).await
                    {
                        eprintln!("Failed to send update via channel: {}", e);
                        break;
                    }
                }

                Err(e) => {
                    eprintln!("Failed to deserialize message: {}, error: {}", text, e);
                }
            }
        }
    }
}

/// Connects to the exchange WebSocket, synchronizes an initial snapshot and
/// processes incremental orderbook updates.
pub async fn connect_ws() -> Result<mpsc::Receiver<WsOrderbookUpdateData>, Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<WsOrderbookUpdateData>(CHANNEL_BUFFER_SIZE);

    let (ws_stream, _) = connect_async(WEBSOCKET_URL).await?;
    println!("WebSocket connected to {}", WEBSOCKET_URL);

    let (write, read) = ws_stream.split();

    // spawn a task to handle incoming WebSocket messages
    tokio::spawn(async move {
        handle_ws_stream(write, read, tx).await;
    });

    Ok(rx)
}