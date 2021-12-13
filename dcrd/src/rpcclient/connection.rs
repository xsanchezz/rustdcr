//! Client connection.
//! Consists all websocket cofigurations.

use crate::dcrjson::types::JsonResponse;

use super::infrastructure::Command;

use {
    super::error::RpcClientError,
    async_trait::async_trait,
    futures_util::stream::SplitSink,
    futures_util::stream::{SplitStream, StreamExt},
    httparse::Status,
    log::info,
    log::warn,
    tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
        sync::mpsc,
    },
    tokio_native_tls::native_tls,
    tokio_tungstenite::{
        tungstenite::{handshake::headers, http::Request, Message},
        MaybeTlsStream, WebSocketStream,
    },
};

#[async_trait]
pub trait RPCConn: Sized + Send + Sync + Clone {
    /// Creates a websocket connection and returns a websocket
    ///  write feeder and a websocket reader. An asynchronous
    /// thread is spawn to forward messages sent from the ws_write feeder.
    async fn ws_split_stream(
        &mut self,
    ) -> Result<(SplitStream<Websocket>, SplitSink<Websocket, Message>), RpcClientError>;
    async fn handle_post_methods(
        &self,
        http_user_command: mpsc::Receiver<Command>,
    ) -> Result<(), RpcClientError>;
    fn is_http_mode(&self) -> bool;
    fn disable_connect_on_new(&self) -> bool;
    fn disable_auto_reconnect(&self) -> bool;
}

/// Describes the connection configuration parameters for the client.
#[derive(Debug, Clone)]
pub struct ConnConfig {
    /// Full websocket url which consists host and port.
    pub host: String,

    /// Username to authenticate to the RPC server.
    pub user: String,

    /// Password to authenticate to the rpc server.
    pub password: String,

    /// Usually specified as `ws`.
    pub endpoint: String,

    /// Strings for a PEM-encoded certificate chain used
    /// for the TLS connection.  It has no effect if the DisableTLS parameter
    /// is true.
    pub certificates: String,

    /// Full socks5 proxy url containing `scheme` usually `Socks5`, `host` and `port` if specified.
    pub proxy_host: Option<String>,

    /// Username to connect to proxy.
    pub proxy_username: String,

    /// Password to connect to proxy.
    pub proxy_password: String,

    /// Specifies whether transport layer security should be
    /// disabled.  It is recommended to always use TLS if the RPC server
    /// supports it as otherwise your username and password is sent across
    /// the wire in cleartext.
    pub disable_tls: bool,

    /// Specifies that a websocket client connection should not be started
    /// when creating the client with `rpcclient::client::new`. Instead, the
    /// client is created and returned unconnected. `Connect` method must be called
    /// to start the websocket.
    pub disable_connect_on_new: bool,

    /// Disable reconnection if websocket fails.
    pub disable_auto_reconnect: bool,

    /// Instructs the client to run using multiple independent
    /// connections issuing HTTP POST requests instead of using the default
    /// of websockets.  Websockets are generally preferred as some of the
    /// features of the client such as notifications only work with websockets,
    /// however, not all servers support the websocket extensions, so this
    /// flag can be set to true to use basic HTTP POST requests instead.
    pub http_post_mode: bool,
}

impl Default for ConnConfig {
    fn default() -> Self {
        ConnConfig {
            certificates: String::new(),
            disable_connect_on_new: false,
            disable_tls: false,
            http_post_mode: false,
            disable_auto_reconnect: false,
            endpoint: String::from("ws"),
            host: "127.0.0.1:19109".to_string(),
            password: String::new(),
            proxy_host: None,
            proxy_username: String::new(),
            proxy_password: String::new(),
            user: String::new(),
        }
    }
}

/// TLS or TCP Websocket connection connection.
pub type Websocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[async_trait]
impl RPCConn for ConnConfig {
    async fn ws_split_stream(
        &mut self,
    ) -> Result<(SplitStream<Websocket>, SplitSink<Websocket, Message>), RpcClientError> {
        let ws = match self.dial_websocket().await {
            Ok(ws) => ws,
            Err(e) => return Err(e),
        };

        // Split websocket to a sink which sends websocket messages to server and a stream which receives websocket messages.
        let (ws_send, ws_rcv) = ws.split();

        Ok((ws_rcv, ws_send))
    }

    async fn handle_post_methods(
        &self,
        mut http_user_command: mpsc::Receiver<Command>,
    ) -> Result<(), RpcClientError> {
        let client = self.create_http_client()?;

        let on_error =
            |err: String, response: JsonResponse, channel: mpsc::Sender<JsonResponse>| async move {
                if let Err(e) = channel.send(response).await {
                    warn!(
                    "({}) Receiving channel closed abruptly on sending error message, error: {}",
                    err, e
                );
                }
            };

        while let Some(cmd) = http_user_command.recv().await {
            let url = if self.disable_tls {
                format!("http://{}", self.host)
            } else {
                format!("https://{}", self.host)
            };

            // Server response.
            let mut json_response = JsonResponse::default();

            let wrapped_request = client
                .post(&url)
                .basic_auth(&self.user, Some(&self.password))
                .body(cmd.rpc_message)
                .build();

            let request = match wrapped_request {
                Ok(e) => e,

                Err(e) => {
                    warn!("Error creating HTTP Post request, error: {}", e);

                    // On error, errors are logged and channel is closed.
                    json_response.error =
                        serde_json::Value::String("Error creating HTTP Post request".to_string());

                    on_error(
                        "HTTP request handshake".to_string(),
                        json_response,
                        cmd.user_channel,
                    )
                    .await;
                    continue;
                }
            };

            let response = match client.execute(request).await {
                Ok(e) => e.bytes().await,

                Err(e) => {
                    warn!("Error sending RPC message to server, error: {}", e);
                    json_response.error = serde_json::Value::String(format!(
                        "Error sending http request, error: {}",
                        e
                    ));

                    on_error(
                        "HTTP request execute".to_string(),
                        json_response,
                        cmd.user_channel,
                    )
                    .await;

                    continue;
                }
            };

            let bytes = match response {
                Ok(e) => e,

                Err(e) => {
                    warn!("Error retrieving HTTP server response, error: {}", e);
                    on_error("HTTP response".to_string(), json_response, cmd.user_channel).await;

                    continue;
                }
            };

            // Marshal server result to a json response.
            json_response = match serde_json::from_slice(&bytes) {
                Ok(m) => m,

                Err(e) => {
                    warn!(
                        "Error unmarshalling binary result, error: {}. \n Message: {:?}",
                        e,
                        std::str::from_utf8(&bytes)
                    );

                    continue;
                }
            };

            let channel = cmd.user_channel;

            if let Err(e) = channel.send(json_response).await {
                warn!(
                    "Receiving request channel closed abruptly on HTTP post mode, error: {}",
                    e
                )
            }
        }

        Ok(())
    }

    fn disable_connect_on_new(&self) -> bool {
        self.disable_connect_on_new
    }

    fn is_http_mode(&self) -> bool {
        self.http_post_mode
    }

    fn disable_auto_reconnect(&self) -> bool {
        self.disable_auto_reconnect
    }
}

impl ConnConfig {
    /// Invokes a websocket stream to rpcclient using optional TLS and socks proxy.
    async fn dial_websocket(
        &mut self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, RpcClientError> {
        let mut buffered_header = Vec::<u8>::new();

        let stream = match self.proxy_host.clone() {
            Some(proxy) => {
                self.add_proxy_header(&mut buffered_header);
                self.connect_stream(proxy.as_str()).await
            }

            None => self.connect_stream(self.host.clone().as_str()).await,
        };

        match stream {
            Ok(mut stream) => {
                if self.proxy_host.is_some() {
                    if let Err(e) = self
                        .dial_connection(&mut buffered_header, &mut stream)
                        .await
                    {
                        return Err(e);
                    }
                }

                let scheme = if self.disable_tls { "ws" } else { "wss" };
                let host = format!("{}://{}/{}", scheme, self.host, self.endpoint);

                let login = format!("{}:{}", self.user, self.password);
                let enc = base64::encode(login.as_bytes());
                let form = format!("Basic {}", enc);

                let wrapped_request = Request::builder()
                    .uri(host)
                    .header("authorization", form)
                    .body(());

                match wrapped_request {
                    Ok(request) => match tokio_tungstenite::client_async(request, stream).await {
                        Ok(websokcet) => Ok(websokcet.0),

                        Err(e) => {
                            warn!("Error creating websocket handshake, error: {}", e);
                            Err(RpcClientError::RpcHandshake(e))
                        }
                    },

                    Err(e) => {
                        warn!("Error building RPC authenticating request, error: {}.", e);

                        Err(RpcClientError::RpcAuthenticationRequest)
                    }
                }
            }

            Err(e) => Err(e),
        }
    }

    /// Upgrades stream connection to a secured layer.
    /// Add to create stream from should be specified in addr parameter.
    async fn connect_stream(
        &mut self,
        addr: &str,
    ) -> Result<MaybeTlsStream<TcpStream>, RpcClientError> {
        let tcp_stream = match tokio::net::TcpStream::connect(addr).await {
            Ok(tcp_stream) => tcp_stream,

            Err(e) => {
                warn!("Error connecting to tcp stream, error: {}", e);
                return Err(RpcClientError::TcpStream(e));
            }
        };

        if self.disable_tls {
            return Ok(MaybeTlsStream::Plain(tcp_stream));
        }

        let mut tls_connector_builder = native_tls::TlsConnector::builder();

        match native_tls::Certificate::from_pem(self.certificates.as_bytes()) {
            Ok(certificate) => {
                // ToDo: check if host name is an ip before accepting invalid hostname.
                tls_connector_builder
                    .add_root_certificate(certificate)
                    .min_protocol_version(native_tls::Protocol::Tlsv12.into())
                    .danger_accept_invalid_certs(true);
            }

            Err(e) => {
                warn!("Error parsing tls certificate, error: {}", e);
                return Err(RpcClientError::WsTlsCertificate(e));
            }
        }

        let wrapped_tls_stream = match tls_connector_builder.build() {
            Ok(tls_connector) => {
                tokio_native_tls::TlsConnector::from(tls_connector)
                    .connect(addr, tcp_stream)
                    .await
            }

            Err(e) => {
                warn!("Error creating tls handshake, error: {}", e);
                return Err(RpcClientError::TlsHandshake(e));
            }
        };

        match wrapped_tls_stream {
            Ok(tls_stream) => Ok(MaybeTlsStream::NativeTls(tls_stream)),

            Err(e) => {
                warn!("Error creating tls stream, error: {}", e);
                Err(RpcClientError::TlsStream(e))
            }
        }
    }

    /// Initiates proxy connection if proxy credentials are specified.
    /// CONNECT header is sent to proxy server using socks5.
    fn add_proxy_header(&mut self, buffered_header: &mut Vec<u8>) {
        buffered_header.extend_from_slice(
            format!(
                "\
            CONNECT {host} HTTP/1.1\r\n\
            Host: {host}\r\n\
            Proxy-Connection: Keep-Alive\r\n",
                host = self.host,
            )
            .as_bytes(),
        );

        // Add Authorization to proxy server passing basic auth credentials to stream header.
        let login = format!("{}:{}", self.user, self.password);

        let mut header_string = String::from("Basic ");
        header_string.push_str(&base64::encode(login.as_str()));

        buffered_header.extend_from_slice(
            format!("{}: {}\r\n", "proxy-authorization", header_string).as_bytes(),
        );

        // Add trailing empty line
        buffered_header.extend_from_slice(b"\r\n");
    }

    /// Dial TCP connection, sending headers.
    async fn dial_connection(
        &self,
        buffered_header: &mut Vec<u8>,
        stream: &mut MaybeTlsStream<TcpStream>,
    ) -> Result<(), RpcClientError> {
        match stream.write_all(buffered_header).await {
            Ok(_) => {}

            Err(e) => {
                warn!(
                    "Error writing request header to proxied stream, error: {}",
                    e
                );
                return Err(RpcClientError::ProxyAuthentication(e));
            }
        };

        let mut read_buffered = Vec::<u8>::new();

        loop {
            match stream.read_to_end(&mut read_buffered).await {
                Ok(_) => {}

                Err(e) => {
                    warn!(
                        "Error reading proxied RPC server received bytes, error: {}.",
                        e
                    );
                    return Err(RpcClientError::ProxyAuthentication(e));
                }
            };
            let mut header_buffer = [httparse::EMPTY_HEADER; headers::MAX_HEADERS];
            let mut response = httparse::Response::new(&mut header_buffer);

            match response.parse(&read_buffered) {
                Ok(val) => match val {
                    Status::Partial => continue,

                    Status::Complete(_) => match response.code {
                        Some(200) => return Ok(()),

                        _ => {
                            warn!(
                                "HTTP status error from proxied websocket, error code: {:?}.",
                                response.code
                            );

                            return Err(RpcClientError::RpcProxyStatus(response.code));
                        }
                    },
                },

                Err(e) => return Err(RpcClientError::RpcProxyResponseParse(e)),
            };
        }
    }

    fn create_http_client(&self) -> Result<reqwest::Client, RpcClientError> {
        let proxy = match self.proxy_host.clone() {
            Some(proxy) => {
                let proxy = reqwest::Proxy::all(proxy);

                let proxy = match proxy {
                    Ok(e) => e,

                    Err(e) => {
                        warn!("Error setting up RPC proxy connection, error: {}", e);
                        return Err(RpcClientError::ProxyConnection);
                    }
                };

                let proxy = if !self.proxy_password.is_empty() || !self.proxy_username.is_empty() {
                    proxy.basic_auth(&self.proxy_username, &self.proxy_password)
                } else {
                    proxy
                };

                Some(proxy)
            }

            None => None,
        };

        let mut request_builder = reqwest::Client::builder();
        request_builder = match proxy {
            Some(e) => request_builder.proxy(e),

            None => request_builder,
        };

        request_builder = match reqwest::Certificate::from_pem(self.certificates.as_bytes()) {
            Ok(certificate) => {
                // ToDo: check if host name is an ip before accepting invalid hostname.
                request_builder
                    .add_root_certificate(certificate)
                    .danger_accept_invalid_certs(true)
            }

            Err(e) => {
                warn!("Error parsing tls certificate, error: {}", e);
                return Err(RpcClientError::HttpTlsCertificate(e));
            }
        };

        let mut headers = reqwest::header::HeaderMap::new();

        let header_value = match reqwest::header::HeaderValue::from_str("application/json") {
            Ok(e) => e,
            Err(e) => {
                warn!(
                    "Failed to set header content type in HTTP Post mode, error: {}",
                    e
                );
                return Err(RpcClientError::HttpHeader(e));
            }
        };

        headers.append(reqwest::header::CONTENT_TYPE, header_value);

        let request_builder = request_builder.default_headers(headers);

        match request_builder.build() {
            Ok(e) => {
                info!("Successful HTTP handshake");
                Ok(e)
            }

            Err(e) => {
                info!("Error building HTTP handshake, error: {}", e);
                Err(RpcClientError::HttpHandshake(e))
            }
        }
    }
}
