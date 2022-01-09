#![allow(missing_docs)]
// ToDo: Add test for RPC proxie and RPC
// Add test for Notification handling
// Add test for notification reregistration
// Add test for commands both on HTTP and normal.

mod conntest {
    use async_trait::async_trait;
    use futures_util::stream::{SplitSink, SplitStream, StreamExt};
    use std::net::TcpListener;
    use std::thread::spawn;
    use tokio::sync::mpsc;
    use tokio_tungstenite::{
        connect_async,
        tungstenite::{
            accept_hdr,
            handshake::{client::Request, server::Response},
            Message,
        },
    };

    use crate::{
        dcrjson::{commands, result_types::JsonResponse},
        rpcclient::{self, connection::Websocket, error::RpcClientError, infrastructure::Command},
    };
    // use serde_json::Value;
    use tokio_tungstenite::tungstenite::error;

    /// Implements JSON RPC request structure to server.
    #[derive(serde::Deserialize)]
    pub struct TestRequest<'a> {
        pub jsonrpc: &'a str,
        pub method: &'a str,
        pub id: u64,
        pub params: Vec<serde_json::Value>,
    }

    #[derive(Clone)]
    struct WebsocketConnTest;

    pub fn _mock_get_block_count(id: u64) -> Message {
        let res = JsonResponse {
            id: serde_json::json!(id),
            method: serde_json::json!(commands::METHOD_GET_BLOCK_COUNT),
            result: serde_json::json!(100),
            params: Vec::new(),
            error: serde_json::Value::Null,
            ..Default::default()
        };

        let marshalled = serde_json::to_string(&res).unwrap();
        Message::Text(marshalled)
    }

    pub fn _start_server() {
        let server = TcpListener::bind("127.0.0.1:3012").unwrap();

        for stream in server.incoming() {
            spawn(move || {
                let callback = |req: &Request, response: Response| {
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
                    let msg = match websocket.read_message() {
                        Ok(msg) => msg,

                        Err(e) => match e {
                            error::Error::ConnectionClosed => return,
                            _ => panic!("connection closed abruptly"),
                        },
                    };
                    if msg.is_binary() || msg.is_text() {
                        let msg_to_str = &msg.to_string();
                        let res: TestRequest = serde_json::from_str(msg_to_str).unwrap();

                        match res.method {
                            commands::METHOD_GET_BLOCK_COUNT => websocket
                                .write_message(_mock_get_block_count(res.id))
                                .unwrap(),
                            _ => unreachable!(),
                        }
                    }
                }
            });
        }
    }

    #[async_trait]
    impl rpcclient::connection::RPCConn for WebsocketConnTest {
        async fn ws_split_stream(
            &mut self,
        ) -> Result<(SplitStream<Websocket>, SplitSink<Websocket, Message>), RpcClientError>
        {
            let (ws_stream, _) = connect_async("ws://127.0.0.1:3012")
                .await
                .expect("Failed to connect");
            println!("WebSocket handshake has been successfully completed");

            let (ws_send, ws_rcv) = ws_stream.split();

            Ok((ws_rcv, ws_send))
        }

        fn disable_connect_on_new(&self) -> bool {
            false
        }

        fn is_http_mode(&self) -> bool {
            false
        }

        fn disable_auto_reconnect(&self) -> bool {
            false
        }

        async fn handle_post_methods(
            &self,
            _http_user_command: mpsc::Receiver<Command>,
        ) -> Result<(), RpcClientError> {
            todo!()
        }
    }

    #[tokio::test]
    async fn test_conn() {
        std::thread::spawn(|| {
            _start_server();
        });

        use crate::rpcclient::{client, notify::NotificationHandlers};

        let mut test_client = client::new(WebsocketConnTest, NotificationHandlers::default())
            .await
            .unwrap();

        test_client.disconnect().await;

        // TODO: Try sending request here.
        match test_client.get_block_count().await.err().unwrap() {
            RpcClientError::RpcDisconnected => println!("client disconnected"),
            e => panic!("rpcclient client not disconnected: {}", e),
        }

        assert!(
            test_client.is_disconnected().await,
            "websocket wasnt disconnected"
        );

        match test_client.connect().await {
            Ok(_) => println!("websocket reconnected"),
            Err(e) => panic!("websocket errored reconnecting: {}", e),
        };

        // TODO: Try sending request here.
        test_client.get_block_count().await.unwrap().await.unwrap();

        // TODO: Try sending request here.
        test_client.shutdown().await;
    }
}
