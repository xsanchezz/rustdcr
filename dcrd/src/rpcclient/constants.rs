#![allow(dead_code)]
/// Time required to retry connecting to websocket.
pub(super) const CONNECTION_RETRY_INTERVAL_SECS: std::time::Duration =
    std::time::Duration::from_secs(10);
/// Number of elements the websocket send channel can queue before blocking.
pub(super) const SEND_BUFFER_SIZE: usize = 50;
/// The required timeframe to send pings to websocket.
pub(super) const KEEP_ALIVE: u64 = 10;
