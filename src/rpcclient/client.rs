use crate::{
    dcrjson::json::JsonID,
    helper::waitgroup,
    rpcclient::{connection, notify},
};

use async_std::{
    future,
    sync::{Arc, Mutex, RwLock},
};
use futures::{channel::mpsc, stream::SplitStream, StreamExt};
use std::{collections::HashMap, error, sync::atomic::AtomicU64};

use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use log::{info, trace, warn};

/// Creates a new RPC client based on the provided connection configuration
/// details.  The notification handlers parameter may be None if you are not
/// interested in receiving notifications and will be ignored if the
/// configuration is set to run in HTTP POST mode.
///
pub async fn new(
    config: connection::ConnConfig,
    notif_handler: notify::NotificationHandlers,
) -> Result<Client, String> {
    todo!()
    // let client = Client {
    //     id: AtomicU64::new(0),

    //     websocket_connection: None,
    //     websocket_receiver_channel: None,

    //     disconnected: RwLock::new(config.disable_connect_on_new),

    //     configuration: Mutex::new(config),

    //     notification_handler: RwLock::new(notif_handler),
    //     notification_state: RwLock::new(notify::NotificationState {
    //         ..Default::default()
    //     }),

    //     connection_established: mpsc::unbounded(),
    //     disconnect: mpsc::unbounded(),
    //     shutdown: mpsc::unbounded(),

    //     waitgroup: waitgroup::WaitGroup::new(),
    // };

    // if client.configuration.lock().await.disable_connect_on_new {
    //     client.start().await
    // }

    // Ok(client)
}

/// Represents a Decred RPC client which allows easy access to the
/// various RPC methods available on a Decred RPC server.  Each of the wrapper
/// functions handle the details of converting the passed and return types to and
/// from the underlying JSON types which are required for the JSON-RPC
/// invocations
///
/// The client provides each RPC in both synchronous (blocking) and asynchronous
/// (non-blocking) forms.  The asynchronous forms are based on the concept of
/// futures where they return an instance of a type that promises to deliver the
/// result of the invocation at some future time.  Invoking the Receive method on
/// the returned future will block until the result is available if it's not
/// already.
///
/// All field in `Client` are async safe.
pub struct Client {
    /// tracks asynchronous requests and is to be updated at realtime.
    id: AtomicU64,

    /// websocket connection to the underlying server, it is protected by a mutex lock.
    websocket_connection: Option<Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>>, // ToDo: This could be expensive.

    websocket_receiver_channel: Option<mpsc::UnboundedReceiver<Message>>,

    /// Holds the connection configuration associated with the client.
    configuration: Mutex<connection::ConnConfig>,

    /// Contains all notification callback functions. It is protected by a mutex lock.
    /// To update notification handlers, you need to call an helper method. ToDo create an helper method.
    notification_handler: Arc<RwLock<notify::NotificationHandlers>>,

    /// Stores current state of notification handlers so that they can be re-registered on
    /// websocket disconnect.
    notification_state: Arc<RwLock<notify::NotificationState>>,

    request_map: Arc<RwLock<HashMap<u64, Vec<u8>>>>,

    /// disconnected indicates whether the client is disconnected from the server.
    disconnected: RwLock<bool>,

    /// connection_established is a network infrastructure that notifies all channel
    /// when the RPC serve is connected or disconnected.
    connection_established: (mpsc::UnboundedSender<bool>, mpsc::UnboundedReceiver<bool>),

    /// disconnect is a channel that calls for disconnection of websocket connection.
    disconnect: (mpsc::UnboundedSender<bool>, mpsc::UnboundedReceiver<bool>),

    /// Broadcast shutdown command to channels so as to disconnect RPC server.
    shutdown: (mpsc::UnboundedSender<bool>, mpsc::UnboundedReceiver<bool>),

    /// Asynchronously blocks.
    waitgroup: waitgroup::WaitGroup,
}

impl Client {
    /// Establishes the initial websocket connection.  This is necessary when
    /// a client was created after setting the DisableConnectOnNew field of the
    // Config struct.
    ///
    /// If the connection fails and retry is true, this method will continue to try
    /// reconnections with backoff until the context is done.
    ///
    /// This method will error if the client is not configured for websockets, if the
    /// connection has already been established, or if none of the connection
    /// attempts were successful. The client will be shut down when the passed
    /// context is terminated.
    pub fn connect(&self) -> Result<(), Box<dyn error::Error>> {
        todo!()
    }

    /// start begins processing input and output messages.
    pub(crate) async fn start_websocket(&self) {
        self.waitgroup.add(1);

        trace!("Starting RPC client");
        let mut config = self.configuration.lock().await;

        let websocket = match config.http_post_mode {
            true => return,

            false => {
                let websokcet = match config.dial_websocket().await {
                    Ok(websocket) => websocket,

                    Err(e) => {
                        warn!("error dialing websocket connection, error: {}", e);
                        return;
                    }
                };

                websokcet
            }
        };

        let (websocket_channel_sender, websocket_channel_receiver) = mpsc::unbounded();

        let (websocket_writer, websocket_reader) = websocket.split();

        // Read websocket messages sent by server and send to client channels.
        let handle_websocket_in =
            Self::handle_websocket_in(websocket_channel_sender.clone(), websocket_reader);

        tokio::spawn(handle_websocket_in);

        let notification_guard = self.notification_handler.read().await;
        notification_guard
            .on_client_connected
            .and_then(|on_client_connected| {
                on_client_connected();
                None::<bool>
            });

        let ws_out_handler = self.websocket_out_handle();
    }

    async fn websocket_out_handle(&self) {}

    /// Tunnels websocket messages received by websocket server from websocket sink to mpsc stream.
    /// Messages received by websocket stream are sent to the mpsc channel whose consumer handles the received
    /// messages and perform notification handling or explicit commands.
    async fn handle_websocket_in(
        websocket_channel_sender: mpsc::UnboundedSender<Message>,
        websocket_read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) {
        let channel_future = websocket_read.for_each(|wrapped_message| async {
            match wrapped_message {
                Ok(message) => match websocket_channel_sender.unbounded_send(message) {
                    // ToDo: Any possibility of websocket being closed???
                    Ok(_) => {}

                    Err(e) => {
                        warn!(
                            "error tunneling websocket message from server, error: {}",
                            e
                        );

                        if websocket_channel_sender.is_closed() {
                            return;
                        }
                    }
                },

                Err(e) => warn!("error getting unwrapped message, error: {}", e),
            }
        });

        channel_future.await;
    }

    /// Maps messages received by websocket to request map.
    /// Message are gotten through websocket producer sink and sent this consumer which then
    /// tunnels websocket message to its unique ID which has been stored in the request mapper.
    /// All function parameters are `consumed` in this function and requires recalling on websocket reconnection
    /// as websocket_consumer becomes non-existent/closed on websocket disconnect.
    async fn handle_message(
        websocket_consumer: mpsc::UnboundedReceiver<Message>,
        request_map: Arc<RwLock<HashMap<u64, mpsc::UnboundedSender<Message>>>>,
    ) {
        let join_future = websocket_consumer.for_each(|message| {
            let clone = request_map.clone();

            let join_handle = async move {
                let guarded_request_map = clone.read().await;

                let mut id = JsonID { id: 0 };

                match &message {
                    Message::Binary(m) => {
                        match serde_json::from_slice(m) {
                            Ok(v) => {
                                id = v;
                            }

                            Err(e) => {
                                warn!("error getting user id while marshalling json, error: {}", e);
                                return;
                            }
                        };
                    }

                    Message::Text(m) => match serde_json::from_str(m) {
                        Ok(v) => {
                            id = v;
                        }

                        Err(e) => {
                            warn!("error getting user id while marshalling json, error: {}", e);
                            return;
                        }
                    },

                    // ToDo: we are to call a reconnect handler if websocket status isn't a 1000
                    // which means Close Success status. Close signal is to be sent to all consumers and producers
                    // so that a new websocket can be created.
                    Message::Close(m) => {}

                    // ToDo: should we be receiving a ping message from server???
                    Message::Ping(m) => {}

                    // ToDo:
                    Message::Pong(m) => {}
                }

                match guarded_request_map.get(&id.id) {
                    Some(mapper) => match mapper.unbounded_send(message) {
                        Ok(_) => {}

                        Err(e) => {
                            warn!("error sending result to consumer, error: {}", e);
                        }
                    },

                    None => warn!(
                        "could not find user consumer in request map on sending websocket message"
                    ),
                }
            };

            tokio::spawn(join_handle);

            future::ready(())
        });

        join_future.await;
    }

    pub fn wait_for_shutdown(&self) {
        self.waitgroup.wait();
    }

    pub fn shutdown(&self) {
        todo!()
        // let mut ws = match &self.websocket_connection {
        //     Some(ws_lock) => match ws_lock.lock() {
        //         Ok(ws) => ws,

        //         Err(e) => {
        //             warn!("error closing websocket on shutdown, err: {}", e);
        //             self.waitgroup.wait();
        //             return;
        //         }
        //     },

        //     None => {
        //         self.waitgroup.wait();
        //         return;
        //     }
        // };

        // match ws.close(Some(tungstenite::protocol::CloseFrame {
        //     reason: "Normal Closure".into(),
        //     code: 1000.into(),
        // })) {
        //     Ok(_) => loop {
        //         match ws.write_pending() {
        //             Ok(_) => {
        //                 break;
        //             }

        //             Err(e) => warn!(
        //                 "error retrieve close success from websocket server, err: {}",
        //                 e
        //             ),
        //         }
        //     },

        //     Err(e) => warn!("error sending close command to websocket, err: {}", e),
        // };

        // self.waitgroup.done();
    }
}
