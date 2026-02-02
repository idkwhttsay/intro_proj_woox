use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{WebSocketStream, connect_async, MaybeTlsStream};
use tokio_tungstenite::tungstenite::{Message, Result, Error};
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use serde_json::json;
use reqwest::{Response, get};
use crate::dto::{OrderbookSnapshot, WsOrderbookUpdate, WsOrderbookUpdateData};
use crate::orderbook::Orderbook;

const WEBSOCKET_URL: &str = "wss://wss.woox.io/v3/public";
const DEPTH: usize = 50;
const TICKER: &str = "PERP_ETH_USDT";
const SNAPSHOT_MAX_LEVEL: usize = 5;

async fn handle_ws_stream(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    tx: mpsc::Sender<WsOrderbookUpdateData>,
) 
{
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
    let (tx, mut rx) = mpsc::channel::<WsOrderbookUpdateData>(10000);

    let (ws_stream, _) = connect_async(WEBSOCKET_URL).await?;
    println!("WebSocket connected to {}", WEBSOCKET_URL);

    let (write, read) = ws_stream.split();

    // spawn a task to handle incoming WebSocket messages
    tokio::spawn(async move {
        handle_ws_stream(write, read, tx).await;
    });

    // initialize orderbook with snapshot
    let mut orderbook = Orderbook::new();
    let snapshot: OrderbookSnapshot = fetch_orderbook_snapshot().await.unwrap();
    orderbook.apply_snapshot(&snapshot);
    println!("Applied initial snapshot. Snapshot ts: {}", snapshot.timestamp);
    println!("{}", orderbook);

    // process incoming orderbook updates
    while let Some(update) = rx.recv().await {
        println!("Received update with ts: {}, prevTs: {}", update.ts, update.prev_ts);
        if orderbook.last_ts() == update.prev_ts {
            // apply update
            orderbook.update(&update);
            println!("{}", orderbook);
        } else if orderbook.last_ts() < update.prev_ts {
            // missed updates, re-fetch snapshot
            let snapshot: OrderbookSnapshot = fetch_orderbook_snapshot().await.unwrap();
            orderbook.apply_snapshot(&snapshot);
            println!("{}", orderbook);
            println!("Applied snapshot due to missing updates. Snapshot ts: {}", snapshot.timestamp);
        } else {
            // old update, ignore
            println!("Received old update. Current last_ts: {}, update prevTs: {}", orderbook.last_ts(), update.prev_ts);
        }
    }

    Err(Box::new(Error::ConnectionClosed))
}