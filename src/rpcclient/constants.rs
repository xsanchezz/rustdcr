pub(crate) const HASH_SIZE: usize = 32;
pub(crate) const CONNECTION_RETRY_INTERVAL_SECS: std::time::Duration =
    std::time::Duration::from_secs(5);

pub(crate) const ERR_COULD_NOT_GENERATE_TLS_HANDSHAKE: &str = "could not generate tls handshake";
pub(crate) const ERR_COULD_NOT_CREATE_TLS_STREAM: &str = "could not create tls stream";
pub(crate) const ERR_COULD_NOT_CREATE_STREAM: &str = "could not create tcp stream";
pub(crate) const ERR_BUILDING_TLS_CERTIFICATE: &str = "error building tls certificate";
pub(crate) const ERR_GENERATING_WEBSOCKET: &str = "error creating websocket";
pub(crate) const ERR_WRITE_STREAM: &str = "error sending header to socket";
pub(crate) const ERR_READ_STREAM: &str = "error reading header from socket";
pub(crate) const ERR_STATUS_CODE: &str = "server replied error code on read stream";
pub(crate) const ERR_PARSING_HEADER_BYTES: &str = "server replied error code on read stream";
pub(crate) const ERR_STARTING_CHANNEL: &str = "error starting mpsc channel";
pub(crate) const ERR_GENERATING_HEADER: &str = "error generating basic authentication header";
pub(crate) const ERR_WRONG_AUTH: &str = "authentication failure";
