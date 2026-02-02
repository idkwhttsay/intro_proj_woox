/// WebSocket URL for connecting to the exchange.
pub const WEBSOCKET_URL: &str = "wss://wss.woox.io/v3/public";
/// Depth of the orderbook to subscribe to via WebSocket.
pub const DEPTH: usize = 50;
/// Ticker symbol for the trading pair.
pub const TICKER: &str = "PERP_ETH_USDT";
/// Buffer size for the WebSocket message channel.
pub const CHANNEL_BUFFER_SIZE: usize = 10000;
/// Maximum levels to request in the orderbook snapshot.
pub const SNAPSHOT_MAX_LEVEL: usize = 5;