use serde::Deserialize;

/// WebSocket envelope for orderbook update messages.
#[derive(Deserialize, Debug)]
pub struct WsOrderbookUpdate {
    pub topic: String,
    pub data: WsOrderbookUpdateData,
}


/// The payload of a websocket orderbook update.
#[derive(Deserialize, Debug)]
pub struct WsOrderbookUpdateData {
    #[serde(rename = "prevTs")]
    pub prev_ts: u64,
    pub bids: Vec<BidAsk>,
    pub asks: Vec<BidAsk>,
    pub ts: u64,
}


/// A single price level (price and quantity) as delivered by the API.
#[derive(Deserialize, Debug)]
pub struct BidAsk {
    pub price: String,
    pub quantity: String,
}


/// REST API snapshot response wrapper.
#[derive(Deserialize, Debug)]
pub struct OrderbookSnapshot {
    pub timestamp: u64,
    pub data: OrderbookSnapshotData,
}


/// The data portion of an `OrderbookSnapshot` containing both sides.
#[derive(Deserialize, Debug)]
pub struct OrderbookSnapshotData {
    pub asks: Vec<BidAsk>,
    pub bids: Vec<BidAsk>,
}