/// RPC client errors
pub enum RpcClientError {
    /// On json marshalling error.
    Marshaller(serde_json::Error),

    /// Invalid authentication to RPC.
    RpcAuthenticationRequest,
    /// Invalid tcp connection to RPC server.
    TcpStream(std::io::Error),
    /// Invalid tls cerificate error.
    TlsCertificate(native_tls::Error),
    /// Invalid tls connection to Server.
    TlsHandshake(native_tls::Error),
    /// Invalid tls connection to RPC server.
    TlsStream(native_tls::Error),
    /// Invalid rpc open command.
    RpcHandshake(tokio_tungstenite::tungstenite::Error),
    /// Failed to send RPC proxy header to proxy server.
    ProxyAuthenticationRequest(std::io::Error),
    /// Proxy server returned an invalid read response on tunnel.
    ProxyAuthenticationResponse(std::io::Error),
    /// Proxy server failed to tunnel RPC server with status code.
    RpcProxyStatus(Option<u16>),
    /// Error parsing response from server.
    RpcProxyResponseParse(httparse::Error),

    /// Websocket already connected to server.
    WebsocketAlreadyConnected,
    /// Client enabled http post mode
    WebsocketDisabled,
}

impl std::fmt::Display for RpcClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RpcClientError::Marshaller(ref err) => write!(f, "Marshaller error: {}.", err),
            RpcClientError::RpcAuthenticationRequest => write!(f, "RPC authentication error."),
            RpcClientError::TcpStream(ref err) => write!(f, "Tcp stream error: {}.", err),
            RpcClientError::TlsCertificate(ref err) => write!(f, "Tls certificate error: {}.", err),
            RpcClientError::TlsHandshake(ref err) => write!(f, "Tls handshake error: {}.", err),
            RpcClientError::TlsStream(ref err) => write!(f, "Tls stream error: {}.", err),
            RpcClientError::RpcHandshake(ref err) => write!(f, "RPC handshake error: {}.", err),
            RpcClientError::ProxyAuthenticationRequest(ref err) => {
                write!(f, "Proxy authentication request error: {}.", err)
            }
            RpcClientError::ProxyAuthenticationResponse(ref err) => {
                write!(f, "Proxy authentication response error: {}.", err)
            }
            RpcClientError::RpcProxyStatus(e) => match e {
                Some(e) => write!(f, "RPC proxy HTTP status error: {}.", e),
                None => write!(f, "RPC proxy HTTP error."),
            },
            RpcClientError::RpcProxyResponseParse(ref err) => {
                write!(f, "RPC proxied reponse error: {}", err)
            }
            RpcClientError::WebsocketAlreadyConnected => {
                write!(f, "Websocket already connected to RPC server.")
            }
            RpcClientError::WebsocketDisabled => {
                write!(f, "Websocket disabled, client using HTTP Post mode.")
            }
        }
    }
}

impl std::fmt::Debug for RpcClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RpcClientError::Marshaller(ref err) => {
                write!(f, "RpcClientError(Marshaller error: {})", err)
            }
            RpcClientError::RpcAuthenticationRequest => {
                write!(f, "RpcClientError(RPC authentication error)")
            }
            RpcClientError::TcpStream(ref err) => {
                write!(f, "RpcClientError(Tcp stream error: {})", err)
            }
            RpcClientError::TlsCertificate(ref err) => {
                write!(f, "RpcClientError(Tls certificate error: {}.)", err)
            }
            RpcClientError::TlsHandshake(ref err) => {
                write!(f, "RpcClientError(Tls handshake error: {})", err)
            }
            RpcClientError::TlsStream(ref err) => {
                write!(f, "RpcClientError(Tls stream error: {})", err)
            }
            RpcClientError::RpcHandshake(ref err) => {
                write!(f, "RpcClientError(RPC handshake error: {})", err)
            }
            RpcClientError::ProxyAuthenticationRequest(ref err) => write!(
                f,
                "RpcClientError(Proxy authentication request error: {})",
                err
            ),
            RpcClientError::ProxyAuthenticationResponse(ref err) => write!(
                f,
                "RpcClientError(Proxy authentication response error: {})",
                err
            ),
            RpcClientError::RpcProxyStatus(ref e) => match e {
                Some(e) => write!(f, "RpcClientError(RPC proxy HTTP status error: {})", e),
                None => write!(f, "RpcClientError(RPC proxy HTTP error)"),
            },
            RpcClientError::RpcProxyResponseParse(ref err) => {
                write!(f, "RpcClientError(RPC proxied reponse error: {})", err)
            }
            RpcClientError::WebsocketAlreadyConnected => {
                write!(f, "Websocket already connected to RPC server")
            }
            RpcClientError::WebsocketDisabled => {
                write!(f, "Websocket disabled, client using HTTP Post mode")
            }
        }
    }
}
