use httparse::Status;
use reqwest::header;
use std::{
    error,
    io::{Read, Write},
    net::TcpStream,
};

use crate::rpcclient::constants;
use tungstenite::{client, WebSocket};

/// Describes the connection configuration parameters for the client.
///
pub struct ConnConfig {
    /// It is full websocket url which consists scheme, host, post
    /// and endpoint.
    pub host: url::Url,

    /// Username to authenticate to the RPC server.
    pub user: String,

    /// Password to authenticate to the rpc server.
    pub password: String,

    /// Specifies whether transport layer security should be
    /// disabled.  It is recommended to always use TLS if the RPC server
    /// supports it as otherwise your username and password is sent across
    /// the wire in cleartext.
    pub disable_tls: bool,

    // Strings for a PEM-encoded certificate chain used
    // for the TLS connection.  It has no effect if the DisableTLS parameter
    // is true.
    pub certificates: String,

    /// Full socks5 proxy url containing scheme, host, port and endpoint if specified.
    pub proxy_host: Option<url::Url>,

    /// Username to connect to proxy.
    pub proxy_username: String,

    /// Password to connect to proxy.
    pub proxy_password: String,

    buffered_header: Vec<u8>,
}

impl ConnConfig {
    /// Initiates a websocket stream to rpcclient using optional TLS and socks proxy.
    ///
    pub fn dial(&mut self) -> Result<WebSocket<client::AutoStream>, String> {
        if self.disable_tls {
            self.host.set_scheme("ws").unwrap();
        } else {
            self.host.set_scheme("wss").unwrap();
        }

        let stream = match self.proxy_host.clone() {
            Some(proxy) => {
                self.add_proxy_header();
                self.connect_stream(proxy)
            }

            None => self.connect_stream(self.host.clone()),
        };

        match stream {
            Ok(mut stream) => {
                self.add_basic_authentication();

                match self.dial_connection(&mut stream) {
                    Ok(_) => match tungstenite::accept(stream) {
                        Ok(websokcet) => return Ok(websokcet),

                        Err(e) => {
                            return Err(error_helper(constants::ERR_GENERATING_WEBSOCKET, e.into()))
                        }
                    },

                    Err(e) => return Err(e.into()),
                };
            }

            Err(e) => return Err(e),
        }
    }

    /// Upgrades stream connection to a secured layer.
    /// Add to create stream from should be specified in addr parameter.
    ///
    fn connect_stream(
        &mut self,
        addr: url::Url,
    ) -> Result<tungstenite::client::AutoStream, String> {
        let tcp_stream = match TcpStream::connect(addr.as_str()) {
            Ok(tcp_stream) => tcp_stream,

            Err(e) => {
                return Err(error_helper(
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
                tls_connector_builder
                    .add_root_certificate(certificate)
                    .min_protocol_version(native_tls::Protocol::Tlsv12.into());
            }

            Err(e) => {
                return Err(error_helper(
                    constants::ERR_BUILDING_TLS_CERTIFICATE,
                    e.into(),
                ))
            }
        }

        let wrapped_tls_stream = match tls_connector_builder.build() {
            Ok(tls_connector) => match addr.domain() {
                Some(domain) => tls_connector.connect(domain, tcp_stream),

                None => return Err(constants::ERR_COULD_NOT_GET_PROXY_DOMAIN.into()),
            },

            Err(e) => {
                return Err(error_helper(
                    constants::ERR_COULD_NOT_GENERATE_TLS_HANDSHAKE,
                    e.into(),
                ));
            }
        };

        match wrapped_tls_stream {
            Ok(tls_stream) => return Ok(tungstenite::stream::Stream::Tls(tls_stream)),

            Err(e) => {
                return Err(error_helper(
                    constants::ERR_COULD_NOT_CREATE_TLS_STREAM,
                    e.into(),
                ));
            }
        }
    }

    /// Adds required basic authentication to RPC server.
    ///
    fn add_basic_authentication(&mut self) {
        let login_credentials = format!("{}:{}", self.user, self.password);

        let mut header_string = String::from("Basic ");
        header_string.push_str(&base64::encode(login_credentials.as_str()));

        self.buffered_header.extend_from_slice(
            &format!("{}: {}\r\n", header::AUTHORIZATION, header_string).as_bytes(),
        );

        // Add trailing empty line
        self.buffered_header.extend_from_slice(b"\r\n");
    }

    /// Initiates proxy connection if proxy credentials are specified. CONNECT
    /// header is sent to proxy server using socks5.
    ///
    fn add_proxy_header(&mut self) {
        self.buffered_header.extend_from_slice(
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

        self.buffered_header.extend_from_slice(
            &format!("{}: {}\r\n", header::PROXY_AUTHORIZATION, header_string).as_bytes(),
        );
    }

    /// Dials stream connection, sending http header to stream.
    fn dial_connection(&self, stream: &mut tungstenite::client::AutoStream) -> Result<(), String> {
        match stream.write_all(&self.buffered_header) {
            Ok(_) => {}

            Err(e) => return Err(error_helper(constants::ERR_WRITE_STREAM, e.into())),
        };

        let mut read_buffered = Vec::<u8>::new();

        loop {
            match stream.read_to_end(&mut read_buffered) {
                Ok(_) => {}

                Err(e) => return Err(error_helper(constants::ERR_READ_STREAM, e.into())),
            };

            let mut header_buffer =
                [httparse::EMPTY_HEADER; tungstenite::handshake::headers::MAX_HEADERS];
            let mut response = httparse::Response::new(&mut header_buffer);

            match response.parse(&read_buffered) {
                Ok(val) => match val {
                    Status::Partial => continue,

                    Status::Complete(_) => match response.code {
                        Some(200) => return Ok(()),

                        _ => {
                            return Err(error_helper(
                                constants::ERR_STATUS_CODE,
                                httparse::Error::Status.into(),
                            ))
                        }
                    },
                },

                Err(e) => return Err(error_helper(constants::ERR_PARSING_HEADER_BYTES, e.into())),
            };
        }
    }
}

fn error_helper(message: &str, explicit_err: Box<dyn error::Error>) -> String {
    let err = format!("{}, err: {}", message, explicit_err);

    return err;
}
// #[cfg(test)]
// mod tests {

//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }

//     #[test]
//     fn ws_dialler() {
//         // let config = ConnConfig {
//         //     host: String::from("localhost:9109"),
//         //     endpoint: String::from("ws"),
//         //     user: String::from("user"),
//         //     password: String::from("password"),
//         // };

//         // dial(&config).unwrap();
//     }
// }
