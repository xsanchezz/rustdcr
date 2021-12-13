#![allow(missing_docs)]
// ToDo: Add test for RPC proxie and RPC
// Add test for Notification handling
// Add test for notification reregistration
// Add test for commands both on HTTP and normal.

// mod test {
//     use async_trait::async_trait;
//     use futures_util::stream::{SplitSink, SplitStream, StreamExt};
//     use std::net::TcpListener;
//     use std::thread::spawn;
//     use tokio_tungstenite::{
//         connect_async,
//         tungstenite::{
//             accept_hdr,
//             handshake::{client::Request, server::Response},
//             Message,
//         },
//     };

//     use crate::{
//         chaincfg::chainhash,
//         rpcclient::{self, client, connection::Websocket, error::RpcClientError},
//     };
//     use serde_json::Value;

//     struct WebsocketConnTest;

//     impl WebsocketConnTest {
//         // pub fn mock_get_block_count(&self) -> serde_json::Value {
//         //     Value::from(1)
//         // }

//         // pub fn mock_get_block_hash(&self) -> serde_json::Value {
//         //     let hash = chainhash::Hash::new(vec![
//         //         0x6f, 0xe2, 0x8c, 0x0a, 0xb6, 0xf1, 0xb3, 0x72, 0xc1, 0xa6, 0xa2, 0x46, 0xae, 0x63,
//         //         0xf7, 0x4f, 0x93, 0x1e, 0x83, 0x65, 0xe1, 0x5a, 0x08, 0x9c, 0x68, 0xd6, 0x19, 0x00,
//         //         0x00, 0x00, 0x00, 0x00,
//         //     ])
//         //     .unwrap();

//         //     Value::from(hash.string().unwrap())
//         // }
//     }

//     impl WebsocketConnTest {
//         pub fn start_server(&self) {
//             let server = TcpListener::bind("127.0.0.1:3012").unwrap();

//             for stream in server.incoming() {
//                 spawn(move || {
//                     let callback = |req: &Request, response: Response| {
//                         println!("Received a new ws handshake");
//                         println!("The request's path is: {}", req.uri().path());
//                         println!("The request's headers are:");
//                         for (ref header, _value) in req.headers() {
//                             println!("* {}", header);
//                         }

//                         // Let's add an additional header to our response to the client.
//                         // let headers = response.headers_mut();
//                         // headers.append("MyCustomHeader", ":)".parse().unwrap());
//                         // headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());

//                         Ok(response)
//                     };

//                     let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

//                     loop {
//                         let msg = websocket.read_message().unwrap();
//                         if msg.is_binary() || msg.is_text() {
//                             println!("{}", msg.to_string());
//                         }
//                     }
//                 });
//             }
//         }
//     }

//     // #[async_trait]
//     // impl rpcclient::connection::WebsocketConn for WebsocketConnTest {
//     //     async fn ws_split_stream(
//     //         &mut self,
//     //     ) -> Result<(SplitStream<Websocket>, SplitSink<Websocket, Message>), RpcClientError>
//     //     {
//     //         self.start_server();

//     //         let (ws_stream, _) = connect_async("127.0.0.1:3012")
//     //             .await
//     //             .expect("Failed to connect");
//     //         println!("WebSocket handshake has been successfully completed");

//     //         let (ws_send, ws_rcv) = ws_stream.split();

//     //         Ok((ws_rcv, ws_send))
//     //     }

//     //     fn disable_connect_on_new(&self) -> bool {
//     //         false
//     //     }

//     //     fn is_http_mode(&self) -> bool {
//     //         false
//     //     }
//     // }

//     // #[tokio::test]
//     // async fn test_conn() {
//     //     use crate::rpcclient::connection::WebsocketConn;

//     //     let mut conn_test = WebsocketConnTest;

//     //     client::new(WebsocketConn, notif_handler)

//     //     let (rcv, send) = conn_test.ws_split_stream().await.unwrap();

//     // }
// }
