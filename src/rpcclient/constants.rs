pub(crate) const HASH_SIZE: usize = 32;

// // Constant errors.
// pub(crate) const ERR_WEBSOCKET_REQUIRED: &str =
//     "a websocket connection is required to use this feature";

pub(crate) const ERR_COULD_NOT_GET_PROXY_DOMAIN: &str = "could not set proxy scheme";
pub(crate) const ERR_COULD_NOT_GENERATE_TLS_HANDSHAKE: &str = "could not generate tls handshake";
pub(crate) const ERR_COULD_NOT_CREATE_TLS_STREAM: &str = "could not create tls stream";
pub(crate) const ERR_COULD_NOT_CREATE_STREAM: &str = "could not create tcp stream";
pub(crate) const ERR_BUILDING_TLS_CERTIFICATE: &str = "error building tls certificate";
pub(crate) const ERR_GENERATING_WEBSOCKET: &str = "error creating websocket";
pub(crate) const ERR_WRITE_STREAM: &str = "error sending header to socket";
pub(crate) const ERR_READ_STREAM: &str = "error reading header from socket";
pub(crate) const ERR_STATUS_CODE: &str = "server replied error code on read stream";
pub(crate) const ERR_PARSING_HEADER_BYTES: &str = "server replied error code on read stream";
