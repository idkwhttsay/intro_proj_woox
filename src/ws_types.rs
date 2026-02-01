use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WsOrderbookUpdate {
    pub topic: String,
    pub ts: u64,
    pub data: WsOrderbookUpdateData,
}


#[derive(Deserialize, Debug)]
pub struct WsOrderbookUpdateData {
    pub s: String,
    pub prevTs: u64,
    pub bids: Vec<BidAsk>,
    pub asks: Vec<BidAsk>,
    pub ts: u64,
}

#[derive(Deserialize, Debug)]
pub struct BidAsk {
    pub price: String,
    pub quantity: String,
}