#![allow(missing_docs)]
// ToDo: Add test for RPC proxie and RPC
// Add test for Notification handling
// Add test for notification reregistration
// Add test for commands both on HTTP and normal.

mod test {
    use async_trait::async_trait;
    use futures_util::stream::{SplitSink, SplitStream};
    use std::net::TcpListener;
    use std::thread::spawn;
    use tokio_tungstenite::tungstenite::{
        accept_hdr,
        handshake::{client::Request, server::Response},
        Message,
    };

    use crate::rpcclient::{self, connection::Websocket, error::RpcClientError};

    struct WebsocketConnTest {}

    impl WebsocketConnTest {
        pub fn start_server(&self) {
            let server = TcpListener::bind("127.0.0.1:3012").unwrap();

            for stream in server.incoming() {
                spawn(move || {
                    let callback = |req: &Request, mut response: Response| {
                        println!("Received a new ws handshake");
                        println!("The request's path is: {}", req.uri().path());
                        println!("The request's headers are:");
                        for (ref header, _value) in req.headers() {
                            println!("* {}", header);
                        }

                        // Let's add an additional header to our response to the client.
                        // let headers = response.headers_mut();
                        // headers.append("MyCustomHeader", ":)".parse().unwrap());
                        // headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());

                        Ok(response)
                    };

                    let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

                    loop {
                        let msg = websocket.read_message().unwrap();
                        if msg.is_binary() || msg.is_text() {
                            println!("{}", msg.to_string());
                        }
                    }
                });
            }
        }
    }

    #[async_trait]
    impl rpcclient::connection::WebsocketConn for WebsocketConnTest {
        async fn ws_split_stream(
            &mut self,
        ) -> Result<(SplitStream<Websocket>, SplitSink<Websocket, Message>), RpcClientError>
        {
            self.start_server();
            todo!()
        }
    }
}
