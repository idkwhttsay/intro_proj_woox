# intro_proj_woox

## Overview

- `src/ws_client.rs` — WebSocket connection, subscribe logic and snapshot
  synchronization.
- `src/dto.rs` — Data transfer objects that mirror API JSON (snapshots and
  websocket updates).
- `src/orderbook.rs` — Simple in-memory orderbook that applies snapshots
  and incremental updates, and prints a readable columnar view.

## Build

```bash
cargo build
```

## Run

```bash
cargo run
```