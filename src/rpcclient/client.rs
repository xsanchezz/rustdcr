use httparse::Status;
use std::{
    error,
    io::{Read, Write},
    net::TcpStream,
    str::FromStr,
};

use tungstenite::{
    client,
    http::{status::StatusCode, Request, Response},
    WebSocket,
};

pub struct ConnConfig {
    pub host: url::Url,

    pub endpoint: String,

    pub user: String,

    pub password: String,

    pub disable_tls: bool,

    pub certificates: String,

    pub proxy: Option<url::Url>,

    pub proxy_username: String,

    pub proxy_password: String,

    buffered_header: Vec<u8>,
}

pub fn new() {}

impl ConnConfig {
    // dial initiates a websocket stream to rpcclient using optional TLS and socks proxy.
    fn dial(mut self) -> Result<WebSocket<client::AutoStream>, Box<dyn error::Error>> {
        let mut stream = if self.proxy.is_none() {
            let stream = self
                .connect_stream(url::Url::from_str(self.host.as_str()).unwrap())
                .unwrap();
            stream
        } else {
            let proxy = self.proxy.clone().unwrap();
            let stream = self
                .connect_stream(url::Url::from_str(proxy.as_str()).unwrap())
                .unwrap();

            self.connect_proxy().unwrap();
            stream
        };

        self.dial_connection(&mut stream).unwrap();

        Ok(tungstenite::accept(stream).unwrap())
        // Add basic authentication.
    }

    // tls_handshake upgrades stream connection to a secured layer.
    // Add to create stream from should be specified in addr parameter.
    fn connect_stream(
        &self,
        addr: url::Url,
    ) -> Result<tungstenite::client::AutoStream, Box<dyn error::Error>> {
        let tcp_stream = TcpStream::connect(addr.host_str().unwrap()).unwrap();

        if self.disable_tls {
            return Ok(tungstenite::stream::Stream::Plain(tcp_stream));
        }

        let mut tls_connector = native_tls::TlsConnector::builder();

        tls_connector
            .add_root_certificate(
                native_tls::Certificate::from_pem(self.certificates.as_bytes()).unwrap(),
            )
            .min_protocol_version(native_tls::Protocol::Tlsv12.into());

        Ok(tungstenite::stream::Stream::Tls(
            tls_connector
                .build()
                .unwrap()
                .connect(addr.host_str().unwrap(), tcp_stream)
                .unwrap(),
        ))
    }

    // add_basic_authentication adds required basic authentication to RPC server.
    fn add_basic_authentication(&mut self) {
        let login_credentials = format!("{}:{}", self.user, self.password);

        let mut header_string = String::from("Basic ");
        header_string.push_str(&base64::encode(login_credentials.as_str()));

        self.buffered_header.extend_from_slice(
            &format!("{}: {}\r\n", reqwest::header::AUTHORIZATION, header_string).as_bytes(),
        );

        // Add trailing empty line
        self.buffered_header.extend_from_slice(b"\r\n");
    }

    // connect_proxy initiates proxy connection if proxy credentials are specified.
    // connect_proxy adds CONNECT header as a request to remote proxy using Socks5.
    fn connect_proxy(&mut self) -> Result<(), Box<dyn error::Error>> {
        self.buffered_header.extend_from_slice(
            format!(
                "\
            CONNECT {host}:{port} HTTP/1.1\r\n\
            Host: {host}:{port}\r\n\
            Proxy-Connection: Keep-Alive\r\n",
                host = self.host,
                port = self.host,
            )
            .as_bytes(),
        );

        // Add Authorization to proxy server passing basic auth credentials to stream header.
        let login = format!("{}:{}", self.user, self.password);

        let mut header_string = String::from("Basic ");
        header_string.push_str(&base64::encode(login.as_str()));

        self.buffered_header.extend_from_slice(
            &format!(
                "{}: {}\r\n",
                reqwest::header::PROXY_AUTHORIZATION,
                header_string
            )
            .as_bytes(),
        );

        Ok(())
    }

    // dial_connection dials stream connection, sending http header to stream.
    fn dial_connection(
        &self,
        stream: &mut tungstenite::client::AutoStream,
    ) -> Result<(), Box<dyn error::Error>> {
        match stream.write_all(&self.buffered_header) {
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        };

        let mut read_buffered = Vec::<u8>::new();

        loop {
            match stream.read_to_end(&mut read_buffered) {
                Ok(_) => {}

                Err(e) => return Err(e.into()),
            };

            let mut header_buffer =
                [httparse::EMPTY_HEADER; tungstenite::handshake::headers::MAX_HEADERS];
            let mut response = httparse::Response::new(&mut header_buffer);

            match response.parse(&read_buffered) {
                Ok(val) => match val {
                    Status::Partial => continue,

                    Status::Complete(_) => match response.code {
                        Some(200) => return Ok(()),

                        _ => return Err(httparse::Error::Status.into()),
                    },
                },

                Err(e) => return Err(e.into()),
            };
        }
    }
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
