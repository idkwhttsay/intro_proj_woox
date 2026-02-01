use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{Message, Result, Error};
use tokio::sync::mpsc;
use serde_json::json;
use reqwest::{Response, get};
use crate::dto::{OrderbookSnapshot, WsOrderbookUpdate, WsOrderbookUpdateData};
use crate::orderbook::Orderbook;

const WEBSOCKET_URL: &str = "wss://wss.woox.io/v3/public";
const DEPTH: usize = 50;
const TICKER: &str = "PERP_ETH_USDT";
const SNAPSHOT_MAX_LEVEL: usize = 5;


/// Fetches the orderbook snapshot from the exchange REST endpoint and deserializes
/// the response into `OrderbookSnapshot`. Returns a `reqwest::Error` on
/// failure.
async fn fetch_orderbook_snapshot() -> Result<OrderbookSnapshot, reqwest::Error> {
    let url = format!(
        "https://api.woox.io/v3/public/orderbook?maxLevel={}&symbol={}",
        SNAPSHOT_MAX_LEVEL, TICKER
    );

    let resp: Response = get(&url).await?;
    let snapshot: OrderbookSnapshot = resp.json().await?;
    Ok(snapshot)
}

/// Connects to the exchange WebSocket, synchronizes an initial snapshot and
/// processes incremental orderbook updates.
pub async fn connect_ws() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel::<WsOrderbookUpdateData>(100);

    let (ws_stream, _) = connect_async(WEBSOCKET_URL).await?;
    println!("WebSocket connected to {}", WEBSOCKET_URL);

    let (mut write, mut read) = ws_stream.split();

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
                        if update.topic.starts_with("orderbookupdate") && let Err(e) = tx.send(update.data).await {
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
    });

    // initialize orderbook with snapshot
    let mut orderbook = Orderbook::new();
    let snapshot: OrderbookSnapshot = fetch_orderbook_snapshot().await.unwrap();
    orderbook.apply_snapshot(&snapshot);
    println!("Applied initial snapshot. Snapshot ts: {}", snapshot.timestamp);
    orderbook.print_columnar();

    // process incoming orderbook updates
    while let Some(update) = rx.recv().await {
        println!("Received update with ts: {}, prevTs: {}", update.ts, update.prevTs);
        if orderbook.last_ts() == update.prevTs {
            // apply update
            orderbook.update(&update);
            orderbook.print_columnar();
        } else if orderbook.last_ts() < update.prevTs {
            // missed updates, re-fetch snapshot
            let snapshot: OrderbookSnapshot = fetch_orderbook_snapshot().await.unwrap();
            orderbook.apply_snapshot(&snapshot);
            orderbook.print_columnar();
            println!("Applied snapshot due to missing updates. Snapshot ts: {}", snapshot.timestamp);
        } else {
            // old update, ignore
            println!("Received old update. Current last_ts: {}, update prevTs: {}", orderbook.last_ts(), update.prevTs);
        }
    }

    Err(Box::new(Error::ConnectionClosed))
}