pub enum Error {
    /// On json marshalling error.
    Marshaller(serde_json::Error),

    /// Invalid authentication to RPC.
    RpcAuthentication,
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Marshaller(ref err) => write!(f, ""),
            Error::RpcAuthentication => write!(f, ""),
            Error::TcpStream(ref err) => write!(f, ""),
            Error::TlsCertificate(ref err) => write!(f, ""),
            Error::TlsHandshake(ref err) => write!(f, ""),
            Error::TlsStream(ref err) => write!(f, ""),
            Error::RpcHandshake(ref err) => write!(f, ""),
            Error::ProxyAuthenticationRequest(ref err) => write!(f, ""),
            Error::ProxyAuthenticationResponse(ref err) => write!(f, ""),
            Error::RpcProxyStatus(e) => match e {
                Some(e) => write!(f, "{}", e),
                None => write!(f, ""),
            },
            Error::RpcProxyResponseParse(ref err) => write!(f, ""),
            Error::WebsocketAlreadyConnected => write!(f, ""),
            Error::WebsocketDisabled => write!(f, ""),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Marshaller(ref err) => write!(f, ""),
            Error::RpcAuthentication => write!(f, ""),
            Error::TcpStream(ref err) => write!(f, ""),
            Error::TlsCertificate(ref err) => write!(f, ""),
            Error::TlsHandshake(ref err) => write!(f, ""),
            Error::TlsStream(ref err) => write!(f, ""),
            Error::RpcHandshake(ref err) => write!(f, ""),
            Error::ProxyAuthenticationRequest(ref err) => write!(f, ""),
            Error::ProxyAuthenticationResponse(ref err) => write!(f, ""),
            Error::RpcProxyStatus(ref e) => match e {
                Some(e) => write!(f, "{}", e),
                None => write!(f, ""),
            },
            Error::RpcProxyResponseParse(ref err) => write!(f, ""),
            Error::WebsocketAlreadyConnected => write!(f, ""),
            Error::WebsocketDisabled => write!(f, ""),
        }
    }
}
