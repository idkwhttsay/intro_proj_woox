use crate::dto::{BidAsk, WsOrderbookUpdateData, OrderbookSnapshot};

/// Represents a single level in the orderbook.
#[derive(Debug)]
struct Level {
    price: f64,
    quantity: f64,
}

/// Represents an orderbook with bids and asks.
pub struct Orderbook {
    bids: Vec<Level>, // sorted descending by price
    asks: Vec<Level>, // sorted ascending by price
    last_ts: u64, // last update timestamp
}

impl Orderbook {
    /// Creates a new, empty Orderbook.
    pub fn new() -> Self {
        Self {
            bids: Vec::new(),
            asks: Vec::new(),
            last_ts: 0,
        }
    }

    /// Applies a full snapshot to the orderbook.
    pub fn apply_snapshot(&mut self, snapshot: &OrderbookSnapshot) {
        self.bids.clear();
        self.asks.clear();
        self.last_ts = snapshot.timestamp;

        for bid in &snapshot.data.bids {
            let (price, quantity) = Self::parse_bid_ask(bid);
            self.bids.push(Level { price, quantity });
        }

        for ask in &snapshot.data.asks {
            let (price, quantity) = Self::parse_bid_ask(ask);
            self.asks.push(Level { price, quantity });
        }

        // sort bids descending and asks ascending
        self.bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
        self.asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

        // keep only top 5 levels
        self.bids.truncate(5);
        self.asks.truncate(5);
    }

    /// Returns the last update timestamp.
    pub fn last_ts(&self) -> u64 {
        self.last_ts
    }

    /// Updates the orderbook with a new WsOrderbookUpdateData.
    pub fn update(&mut self, update: &WsOrderbookUpdateData) {
        self.last_ts = update.ts;

        for bid in &update.bids {
            self.update_level(bid, true);
        }

        for ask in &update.asks {
            self.update_level(ask, false);
        }
    }

    /// Updates a single level in the orderbook.
    fn update_level(&mut self, level_data: &BidAsk, is_bid: bool) {
        // parse price and quantity
        let (price, quantity) = Self::parse_bid_ask(level_data);

        // determine which side to update
        let levels = if is_bid {
            &mut self.bids
        } else {
            &mut self.asks
        };

        if quantity == 0.0 {
            // remove level
            levels.retain(|level| level.price != price);
        } else {
            match levels.iter_mut().find(|level| level.price == price) {
                Some(level) => {
                    // update existing level
                    level.quantity = quantity;
                },
                None => {
                    // insert new level
                    levels.push(Level { price, quantity });
                    if is_bid {
                        levels.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
                    } else {
                        levels.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                    }

                    // keep only 5 best levels for bids and asks
                    levels.truncate(5);
                }
            }
        }
    }

    /// Prints the orderbook in a columnar format.
    pub fn print_columnar(&self) {
        println!("{:<20} | {:<20}", "Bids", "Asks");
        println!("{:-<20}-+-{:-<20}", "", "");

        let max_levels = self.bids.len().max(self.asks.len());

        for i in 0..max_levels {
            let bid_str = if i < self.bids.len() {
                format!("{:.2} ({:.4})", self.bids[i].price, self.bids[i].quantity)
            } else {
                String::new()
            };

            let ask_str = if i < self.asks.len() {
                format!("{:.2} ({:.4})", self.asks[i].price, self.asks[i].quantity)
            } else {
                String::new()
            };

            println!("{:<20} | {:<20}", bid_str, ask_str);
        }
    }

    fn parse_bid_ask(level_data: &BidAsk) -> (f64, f64) {
        let price: f64 = level_data.price.parse().unwrap_or(0.0);
        let quantity: f64 = level_data.quantity.parse().unwrap_or(0.0);
        (price, quantity)
    }
}