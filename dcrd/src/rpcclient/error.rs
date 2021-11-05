//! Contains all RPC client errors.
use {thiserror::Error, tokio_native_tls::native_tls};
/// RPC client errors
#[derive(Error, Debug)]
pub enum RpcClientError {
    /// On json marshalling error.
    #[error("marshaller error: {0}")]
    Marshaller(serde_json::Error),

    /// Unregisted on server notification callback.
    #[error("unregistered notification callback, type: {0}")]
    UnregisteredNotification(String),
    /// Invalid authentication to RPC.
    #[error("rpc authentication error")]
    RpcAuthenticationRequest,
    /// Invalid tcp connection to RPC server.
    #[error("tcp stream error: {0}")]
    TcpStream(std::io::Error),
    /// Invalid tls cerificate error on websocket.
    #[error("websocket tls certificate error: {0}")]
    WsTlsCertificate(native_tls::Error),
    /// Invalid tls connection to Server.
    #[error("tls handshake error: {0}")]
    TlsHandshake(native_tls::Error),
    /// Invalid tls connection to RPC server.
    #[error("tls stream error: {0}")]
    TlsStream(native_tls::Error),
    /// Invalid rpc open command.
    #[error("rpc handshake error: {0}")]
    RpcHandshake(tokio_tungstenite::tungstenite::Error),
    /// Invalid proxy connection
    #[error("invalid proxy connection")]
    ProxyConnection,
    /// Failed to set proxy authentication.
    #[error("proxy authentication request error: {0}")]
    ProxyAuthentication(std::io::Error),
    /// Proxy server failed to tunnel RPC server with status code.
    #[error("rpc proxy http status error: {0:?}")]
    RpcProxyStatus(Option<u16>),
    /// Error parsing response from server.
    #[error("rpc proxied reponse error: {0}")]
    RpcProxyResponseParse(httparse::Error),
    /// Websocket RPC disconnection from server.
    #[error("rpc client disconnected")]
    RpcDisconnected,

    /// Websocket already connected to server.
    #[error("websocket already connected to RPC server")]
    WebsocketAlreadyConnected,
    /// Client enabled http post mode
    #[error("websocket disabled, client using HTTP Post mode")]
    ClientNotConnected,
}
