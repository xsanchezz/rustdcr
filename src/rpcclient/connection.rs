use httparse::Status;
use reqwest::header;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{helper::error_helper, rpcclient::constants};
use tungstenite::{client, WebSocket};

/// Describes the connection configuration parameters for the client.
#[derive(Debug)]
pub struct ConnConfig {
    /// It is full websocket url which consists host and port.
    pub host: String,

    /// Username to authenticate to the RPC server.
    pub user: String,

    /// Password to authenticate to the rpc server.
    pub password: String,

    /// Usually specified as `ws`.
    pub endpoint: String,

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
}

impl Default for ConnConfig {
    fn default() -> Self {
        ConnConfig {
            certificates: String::new(),
            disable_connect_on_new: false,
            disable_tls: false,
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

impl ConnConfig {
    /// Invokes a websocket stream to rpcclient using optional TLS and socks proxy.
    pub fn dial_websocket(&mut self) -> Result<WebSocket<client::AutoStream>, String> {
        let mut buffered_header = Vec::<u8>::new();

        let stream = match self.proxy_host.clone() {
            Some(proxy) => {
                self.add_proxy_header(&mut buffered_header);
                self.connect_stream(proxy.as_str())
            }

            None => self.connect_stream(self.host.clone().as_str()),
        };

        match stream {
            Ok(mut stream) => {
                if self.proxy_host.is_some() {
                    match self.dial_connection(&mut buffered_header, &mut stream) {
                        Ok(()) => {}

                        Err(e) => return Err(e.into()),
                    }
                }

                let scheme = if self.disable_tls { "ws" } else { "wss" };
                let host = format!("{}://{}/{}", scheme, self.host, self.endpoint);

                let login = format!("{}:{}", self.user, self.password);
                let enc = base64::encode(login.as_bytes());
                let form = format!("Basic {}", enc);

                let wrapped_request = tungstenite::handshake::client::Request::builder()
                    .uri(host)
                    .header(header::AUTHORIZATION, form)
                    .body(());

                match wrapped_request {
                    Ok(request) => {
                        match tungstenite::client(request, stream) {
                            Ok(websokcet) => {
                                return Ok(websokcet.0);
                            }

                            // ToDo: cast error to get http status code.
                            Err(e) => {
                                return Err(error_helper::new(
                                    constants::ERR_GENERATING_WEBSOCKET,
                                    e.into(),
                                ));
                            }
                        };
                    }

                    Err(e) => {
                        return Err(error_helper::new(
                            constants::ERR_GENERATING_HEADER,
                            e.into(),
                        ))
                    }
                }
            }

            Err(e) => return Err(e),
        }
    }

    /// Upgrades stream connection to a secured layer.
    /// Add to create stream from should be specified in addr parameter.
    fn connect_stream(&mut self, addr: &str) -> Result<tungstenite::client::AutoStream, String> {
        let tcp_stream = match TcpStream::connect(addr) {
            Ok(tcp_stream) => tcp_stream,

            Err(e) => {
                return Err(error_helper::new(
                    constants::ERR_COULD_NOT_CREATE_STREAM,
                    e.into(),
                ))
            }
        };

        if self.disable_tls {
            return Ok(tungstenite::stream::Stream::Plain(tcp_stream));
        }

        let mut tls_connector_builder = native_tls::TlsConnector::builder();

        match native_tls::Certificate::from_pem(self.certificates.as_bytes()) {
            Ok(certificate) => {
                // ToDo: check if host name is an ip before accepting invalid hostname.
                tls_connector_builder
                    .add_root_certificate(certificate)
                    .min_protocol_version(native_tls::Protocol::Tlsv12.into())
                    .danger_accept_invalid_hostnames(true)
                    .danger_accept_invalid_certs(true);
            }

            Err(e) => {
                return Err(error_helper::new(
                    constants::ERR_BUILDING_TLS_CERTIFICATE,
                    e.into(),
                ))
            }
        }

        let wrapped_tls_stream = match tls_connector_builder.build() {
            Ok(tls_connector) => tls_connector.connect(addr, tcp_stream),

            Err(e) => {
                return Err(error_helper::new(
                    constants::ERR_COULD_NOT_GENERATE_TLS_HANDSHAKE,
                    e.into(),
                ));
            }
        };

        match wrapped_tls_stream {
            Ok(tls_stream) => return Ok(tungstenite::stream::Stream::Tls(tls_stream)),

            Err(e) => {
                return Err(error_helper::new(
                    constants::ERR_COULD_NOT_CREATE_TLS_STREAM,
                    e.into(),
                ));
            }
        }
    }

    /// Initiates proxy connection if proxy credentials are specified. CONNECT header is sent
    /// to proxy server using socks5.
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
            &format!("{}: {}\r\n", header::PROXY_AUTHORIZATION, header_string).as_bytes(),
        );

        // Add trailing empty line
        buffered_header.extend_from_slice(b"\r\n");
    }

    /// Dials stream connection, sending http header to stream if user is
    /// using a proxy server for websocket connection.
    fn dial_connection(
        &self,
        buffered_header: &mut Vec<u8>,
        stream: &mut tungstenite::client::AutoStream,
    ) -> Result<(), String> {
        match stream.write_all(buffered_header) {
            Ok(_) => {}

            Err(e) => return Err(error_helper::new(constants::ERR_WRITE_STREAM, e.into())),
        };

        let mut read_buffered = Vec::<u8>::new();

        loop {
            match stream.read_to_end(&mut read_buffered) {
                Ok(_) => {}

                Err(e) => return Err(error_helper::new(constants::ERR_READ_STREAM, e.into())),
            };

            let mut header_buffer =
                [httparse::EMPTY_HEADER; tungstenite::handshake::headers::MAX_HEADERS];
            let mut response = httparse::Response::new(&mut header_buffer);

            match response.parse(&read_buffered) {
                Ok(val) => match val {
                    Status::Partial => continue,

                    Status::Complete(_) => match response.code {
                        Some(200) => return Ok(()),

                        Some(401) => {
                            return Err(error_helper::new(
                                constants::ERR_WRONG_AUTH,
                                httparse::Error::Status.into(),
                            ))
                        }

                        _ => {
                            return Err(error_helper::new(
                                constants::ERR_STATUS_CODE,
                                httparse::Error::Status.into(),
                            ))
                        }
                    },
                },

                Err(e) => {
                    return Err(error_helper::new(
                        constants::ERR_PARSING_HEADER_BYTES,
                        e.into(),
                    ))
                }
            };
        }
    }
}
