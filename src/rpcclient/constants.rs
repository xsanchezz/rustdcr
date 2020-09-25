pub(super) const HASH_SIZE: usize = 32;
pub(super) const CONNECTION_RETRY_INTERVAL_SECS: std::time::Duration =
    std::time::Duration::from_secs(10);

pub(super) const ERR_COULD_NOT_GENERATE_TLS_HANDSHAKE: &str = "could not generate tls handshake";
pub(super) const ERR_COULD_NOT_CREATE_TLS_STREAM: &str = "could not create tls stream";
pub(super) const ERR_COULD_NOT_CREATE_STREAM: &str = "could not create tcp stream";
pub(super) const ERR_BUILDING_TLS_CERTIFICATE: &str = "error building tls certificate";
pub(super) const ERR_GENERATING_WEBSOCKET: &str = "error creating websocket";
pub(super) const ERR_WRITE_STREAM: &str = "error sending header to socket";
pub(super) const ERR_READ_STREAM: &str = "error reading header from socket";
pub(super) const ERR_STATUS_CODE: &str = "server replied error code on read stream";
pub(super) const ERR_PARSING_HEADER_BYTES: &str = "server replied error code on read stream";
// pub(super) const ERR_STARTING_CHANNEL: &str = "error starting mpsc channel";
pub(super) const ERR_GENERATING_HEADER: &str = "error generating basic authentication header";
pub(super) const ERR_WRONG_AUTH: &str = "authentication failure";

/// SEND_BUFFER_SIZE is the number of elements the websocket send channel
/// can queue before blocking.
pub(super) const SEND_BUFFER_SIZE: usize = 50;
/// The required timeframe to send pings to websocket.
pub(super) const KEEP_ALIVE: u64 = 5;
