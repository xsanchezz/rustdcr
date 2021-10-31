// /// RPC client errors
// pub enum RpcClientError {
//     /// On json marshalling error.
//     Marshaller(serde_json::Error),

//     /// Unregisted on server notification callback.
//     UnregisteredNotification(String),
//     /// Invalid authentication to RPC.
//     RpcAuthenticationRequest,
//     /// Invalid tcp connection to RPC server.
//     TcpStream(std::io::Error),
//     /// Invalid tls cerificate error on websocket.
//     WsTlsCertificate(native_tls::Error),
//     /// Invalid tls connection to Server.
//     TlsHandshake(native_tls::Error),
//     /// Invalid tls connection to RPC server.
//     TlsStream(native_tls::Error),
//     /// Invalid rpc open command.
//     RpcHandshake(tokio_tungstenite::tungstenite::Error),
//     /// Invalid proxy connection
//     ProxyConnection,
//     /// Failed to set proxy authentication.
//     ProxyAuthentication(std::io::Error),
//     /// Proxy server failed to tunnel RPC server with status code.
//     RpcProxyStatus(Option<u16>),
//     /// Error parsing response from server.
//     RpcProxyResponseParse(httparse::Error),
//     /// Websocket RPC disconnection from server.
//     RpcDisconnected,

//     /// Error setting http header
//     HttpHeader(reqwest::header::InvalidHeaderValue),
//     /// Invalid http handshake to server.
//     HttpHandshake(reqwest::Error),
//     /// Invalid tls cerificate error on HTTP.
//     HttpTlsCertificate(reqwest::Error),

//     /// Websocket already connected to server.
//     WebsocketAlreadyConnected,
//     /// Client enabled http post mode
//     ClientNotConnected,
// }

// impl std::fmt::Display for RpcClientError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match *self {
//             RpcClientError::Marshaller(ref err) => write!(f, "Marshaller error: {}.", err),
//             RpcClientError::UnregisteredNotification(ref e) => {
//                 write!(f, "Unregistered notification callback, type: {}", e)
//             }
//             RpcClientError::RpcAuthenticationRequest => write!(f, "RPC authentication error."),
//             RpcClientError::TcpStream(ref err) => write!(f, "Tcp stream error: {}.", err),
//             RpcClientError::WsTlsCertificate(ref err) => {
//                 write!(f, "Websocket tls certificate error: {}.", err)
//             }
//             RpcClientError::TlsHandshake(ref err) => write!(f, "Tls handshake error: {}.", err),
//             RpcClientError::TlsStream(ref err) => write!(f, "Tls stream error: {}.", err),
//             RpcClientError::RpcHandshake(ref err) => write!(f, "RPC handshake error: {}.", err),
//             RpcClientError::ProxyAuthentication(ref err) => {
//                 write!(f, "Proxy authentication request error: {}.", err)
//             }
//             RpcClientError::ProxyConnection => write!(f, "Invalid proxy connection."),
//             RpcClientError::RpcProxyStatus(e) => match e {
//                 Some(e) => write!(f, "RPC proxy HTTP status error: {}.", e),
//                 None => write!(f, "RPC proxy HTTP error."),
//             },
//             RpcClientError::RpcProxyResponseParse(ref err) => {
//                 write!(f, "RPC proxied reponse error: {}", err)
//             }
//             RpcClientError::RpcDisconnected => write!(f, "RPC client disconnected."),
//             RpcClientError::HttpHeader(ref e) => write!(
//                 f,
//                 "Failed to set HTTP header on HTTP Post mode, error: {}.",
//                 e
//             ),
//             RpcClientError::HttpHandshake(ref e) => write!(
//                 f,
//                 "Error initiating HTTP Hanshake in HTTP Post mode, error: {}.",
//                 e
//             ),
//             RpcClientError::HttpTlsCertificate(ref e) => {
//                 write!(f, "HTTP certificate error: {}.", e)
//             }
//             RpcClientError::WebsocketAlreadyConnected => {
//                 write!(f, "Websocket already connected to RPC server.")
//             }
//             RpcClientError::ClientNotConnected => {
//                 write!(f, "Websocket disabled, client using HTTP Post mode.")
//             }
//         }
//     }
// }

// impl std::fmt::Debug for RpcClientError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match *self {
//             RpcClientError::Marshaller(ref err) => {
//                 write!(f, "RpcClientError(Marshaller error: {})", err)
//             }
//             RpcClientError::UnregisteredNotification(ref e) => write!(
//                 f,
//                 "RpcClientError(Unregistered notification callback, type: {})",
//                 e
//             ),
//             RpcClientError::RpcAuthenticationRequest => {
//                 write!(f, "RpcClientError(RPC authentication error)")
//             }
//             RpcClientError::TcpStream(ref err) => {
//                 write!(f, "RpcClientError(Tcp stream error: {})", err)
//             }
//             RpcClientError::WsTlsCertificate(ref err) => write!(
//                 f,
//                 "RpcClientError(Websocket tls certificate error: {}.)",
//                 err
//             ),
//             RpcClientError::TlsHandshake(ref err) => {
//                 write!(f, "RpcClientError(Tls handshake error: {})", err)
//             }
//             RpcClientError::TlsStream(ref err) => {
//                 write!(f, "RpcClientError(Tls stream error: {})", err)
//             }
//             RpcClientError::RpcHandshake(ref err) => {
//                 write!(f, "RpcClientError(RPC handshake error: {})", err)
//             }
//             RpcClientError::ProxyAuthentication(ref err) => write!(
//                 f,
//                 "RpcClientError(Proxy authentication request error: {})",
//                 err
//             ),
//             RpcClientError::ProxyConnection => {
//                 write!(f, "RpcClientError(Invalid proxy connection)")
//             }
//             RpcClientError::RpcProxyStatus(ref e) => match e {
//                 Some(e) => write!(f, "RpcClientError(RPC proxy HTTP status error: {})", e),
//                 None => write!(f, "RpcClientError(RPC proxy HTTP error)"),
//             },
//             RpcClientError::RpcProxyResponseParse(ref err) => {
//                 write!(f, "RpcClientError(RPC proxied reponse error: {})", err)
//             }
//             RpcClientError::RpcDisconnected => write!(f, "RpcClientError(RPC client disconnected)"),
//             RpcClientError::HttpHeader(ref e) => write!(
//                 f,
//                 "RpcClientError(Failed to set HTTP header on HTTP Post mode, error: {})",
//                 e
//             ),
//             RpcClientError::HttpHandshake(ref e) => write!(
//                 f,
//                 "RpcClientError(Error initiating HTTP Hanshake in HTTP Post mode, error: {})",
//                 e
//             ),
//             RpcClientError::HttpTlsCertificate(ref e) => {
//                 write!(f, "RpcClientError(HTTP certificate error: {})", e)
//             }
//             RpcClientError::WebsocketAlreadyConnected => write!(
//                 f,
//                 "RpcClientError(Websocket already connected to RPC server)"
//             ),
//             RpcClientError::ClientNotConnected => write!(
//                 f,
//                 "RpcClientError(Websocket disabled, client using HTTP Post mode)"
//             ),
//         }
//     }
// }
