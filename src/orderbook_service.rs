use crate::{dto::OrderbookSnapshot, orderbook::Orderbook, ws_client};
use reqwest::{Response, get};
use crate::consts::{SNAPSHOT_MAX_LEVEL, TICKER};

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

pub async fn initialize_orderbook() -> Orderbook {
    // initialize orderbook with snapshot
    let mut orderbook = Orderbook::new();
    let snapshot: OrderbookSnapshot = fetch_orderbook_snapshot().await.unwrap();
    orderbook.apply_snapshot(&snapshot);
    println!("Applied initial snapshot. Snapshot ts: {}", snapshot.timestamp);
    println!("{}", orderbook);
    
    orderbook
}

pub async fn run_orderbook_service() {
    let mut orderbook = initialize_orderbook().await;

    let mut rx = ws_client::connect_ws().await.unwrap();

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
}