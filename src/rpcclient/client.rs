use super::{connection, constants, infrastructure, notify};
use crate::helper::waitgroup;

use log::{info, warn};

use async_std::sync::{Arc, Mutex, RwLock};

use futures::stream::SplitStream;
use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::AtomicU64,
};

use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub type Websocket = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Creates a new RPC client based on the provided connection configuration
/// details.  The notification handlers parameter may be None if you are not
/// interested in receiving notifications and will be ignored if the
/// configuration is set to run in HTTP POST mode.
pub async fn new(
    config: connection::ConnConfig,
    notif_handler: notify::NotificationHandlers,
) -> Result<Client, String> {
    let (ws_rcv_chan_send, ws_rcv_chan_rcv) = mpsc::channel(constants::SEND_BUFFER_SIZE);
    let (disconnect_ws_send, disconnect_ws_rcv) = mpsc::channel(1);
    let ws_disconnect_acknowledgement = mpsc::channel(1);

    let mut client = Client {
        _id: AtomicU64::new(0),
        configuration: Arc::new(Mutex::new(config)),
        disconnect_ws: disconnect_ws_send,
        is_ws_disconnected: Arc::new(RwLock::new(true)),
        _notification_handler: Arc::new(notif_handler),
        _notification_state: Arc::new(RwLock::new(notify::NotificationState::default())),
        receiver_channel_id_mapper: Arc::new(Mutex::new(HashMap::new())),
        requests_queue_container: Arc::new(Mutex::new(VecDeque::new())),
        user_command: ws_rcv_chan_send,
        ws_disconnected_acknowledgement: ws_disconnect_acknowledgement.1,
        waitgroup: waitgroup::new(),
    };

    let config = client.configuration.clone();
    let mut config = config.lock().await;

    client.waitgroup.add(1);

    if !config.disable_connect_on_new && !config.http_post_mode {
        println!("establishing websocket connection");

        match config.ws_split_stream().await {
            Ok(ws) => {
                client
                    .ws_handler(
                        ws_rcv_chan_rcv,
                        disconnect_ws_rcv,
                        ws_disconnect_acknowledgement.0,
                        ws,
                    )
                    .await;

                *client.is_ws_disconnected.write().await = false;
            }

            Err(e) => return Err(e),
        };
    }

    Ok(client)
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
    _id: AtomicU64,

    /// A channel that tunnels converted users messages to ws_write_middleman to be consumed by websocket writer.
    user_command: mpsc::Sender<infrastructure::Command>, // ToDo: not needed

    /// A channel that calls for disconnection of websocket connection.
    disconnect_ws: mpsc::Sender<()>,

    /// A channel that acknowledges websocket disconnection.
    ws_disconnected_acknowledgement: mpsc::Receiver<()>,

    /// Holds the connection configuration associated with the client.
    configuration: Arc<Mutex<connection::ConnConfig>>,

    /// Contains all notification callback functions. It is protected by a mutex lock.
    /// To update notification handlers, you need to call an helper method. ToDo create an helper method.
    _notification_handler: Arc<notify::NotificationHandlers>,

    /// Stores current state of notification handlers so that they can be re-registered on
    /// websocket disconnect.
    _notification_state: Arc<RwLock<notify::NotificationState>>,

    /// Stores all requests to be be sent to the RPC server.
    requests_queue_container: Arc<Mutex<VecDeque<Message>>>,

    /// Maps request ID to receiver channel.
    /// Messages received from rpc server are mapped with ID stored.
    receiver_channel_id_mapper: Arc<Mutex<HashMap<u64, mpsc::Sender<Message>>>>,

    /// Indicates whether the client is disconnected from the server.
    is_ws_disconnected: Arc<RwLock<bool>>,

    /// Asynchronously blocks.
    waitgroup: waitgroup::WaitGroup,
}

impl Client {
    /// Handles websocket connection to server by calling selective function to handle websocket send, write and reconnect.
    ///
    /// `user_command` is a receiving channel that channels processed RPC command from client.
    ///
    /// `disconnect_ws_cmd_rcv` is a channel that receives websocket disconnect from client.
    ///
    /// `ws_disconnect_acknowledgement` is a channel that sends websocket disconnect success message back to client.
    ///
    /// `split_stream` is a tuple that contains websocket stream for reading websocket messages and a channel to tunnel messages
    /// to websocket writer `Sink`.
    ///
    /// All websocket connection is implemented in this function and all child functions are spawned asynchronously.
    pub(self) async fn ws_handler(
        &mut self,
        user_command: mpsc::Receiver<infrastructure::Command>,
        disconnect_ws_cmd_rcv: mpsc::Receiver<()>,
        ws_disconnect_acknowledgement: mpsc::Sender<()>,
        split_stream: (Websocket, mpsc::Sender<Message>),
    ) {
        self.waitgroup.add(1);

        let new_ws_writer = mpsc::channel(1);

        let queue_command = mpsc::channel(1);

        let msg_acknowledgement = mpsc::channel(1);

        let request_queue_update = mpsc::channel(1);

        let websocket_out = infrastructure::handle_websocket_out(
            split_stream.1,
            new_ws_writer.1,
            queue_command.1,
            msg_acknowledgement.0,
            request_queue_update.1,
            disconnect_ws_cmd_rcv,
        );

        let handle_rcvd_msg = mpsc::unbounded_channel();

        let new_ws_reader = mpsc::channel(1);

        let signal_ws_reconnect = mpsc::channel(1);

        let websocket_in = infrastructure::handle_websocket_in(
            handle_rcvd_msg.0,
            split_stream.0,
            new_ws_reader.1,
            signal_ws_reconnect.0,
        );

        let rcvd_msg_handler = infrastructure::handle_received_message(
            handle_rcvd_msg.1,
            ws_disconnect_acknowledgement,
            self.receiver_channel_id_mapper.clone(),
        );

        let ws_write_middleman = infrastructure::ws_write_middleman(
            user_command,
            request_queue_update.0,
            msg_acknowledgement.1,
            queue_command.0,
            self.requests_queue_container.clone(),
            self.receiver_channel_id_mapper.clone(),
        );

        let on_client_connected = self
            ._notification_handler
            .on_client_connected
            .unwrap_or(|| {});

        let reconnect_handler = infrastructure::ws_reconnect_handler(
            self.configuration.clone(),
            self.is_ws_disconnected.clone(),
            signal_ws_reconnect.1,
            new_ws_reader.0,
            new_ws_writer.0,
            on_client_connected,
        );

        // Separately spawn asynchronous thread for each instances.
        tokio::spawn(websocket_out);
        tokio::spawn(websocket_in);
        tokio::spawn(rcvd_msg_handler);
        tokio::spawn(ws_write_middleman);
        tokio::spawn(reconnect_handler);

        on_client_connected();

        self.waitgroup.done();
    }

    /// Establishes the initial websocket connection.  This is necessary when a client was
    /// created after setting the DisableConnectOnNew field of the Config struct.
    ///
    /// If the connection fails and retry is true, this method will continue to try reconnections
    /// with backoff until the context is done.
    ///
    /// This method will error if the client is not configured for websockets, if the
    /// connection has already been established, or if none of the connection
    /// attempts were successful. The client will be shut down when the passed
    /// context is terminated.
    pub async fn connect(&mut self) -> Result<(), String> {
        if !*self.is_ws_disconnected.read().await {
            return Ok(());
        }

        let mut config = self.configuration.lock().await;
        if config.http_post_mode {
            return Err("Not websocket".into());
        }

        let mut is_ws_disconnected = self.is_ws_disconnected.write().await;
        if *is_ws_disconnected == false {
            return Err("Already connected".into());
        }

        let user_command_channel = mpsc::channel(1);
        let disconnect_ws_channel = mpsc::channel(1);
        let ws_disconnect_acknowledgement = mpsc::channel(1);

        self.user_command = user_command_channel.0;
        self.disconnect_ws = disconnect_ws_channel.0;
        self.ws_disconnected_acknowledgement = ws_disconnect_acknowledgement.1;

        let ws = match config.ws_split_stream().await {
            Ok(ws) => ws,

            Err(e) => return Err(e),
        };

        *is_ws_disconnected = false;

        drop(config);
        drop(is_ws_disconnected);

        self.ws_handler(
            user_command_channel.1,
            disconnect_ws_channel.1,
            ws_disconnect_acknowledgement.0,
            ws,
        )
        .await;

        Ok(())
    }

    /// Disconnects RPC server, deletes command queue and errors any pending request by client.
    pub async fn disconnect(&mut self) {
        // Return if websocket is disconnected.
        let mut is_ws_disconnected = self.is_ws_disconnected.write().await;
        if *is_ws_disconnected == true {
            return;
        }

        *is_ws_disconnected = true;

        drop(is_ws_disconnected);

        if self.disconnect_ws.send(()).await.is_err() {
            warn!("error sending disconnect command to webserver, disconnect_ws closed.")
        }

        if self.ws_disconnected_acknowledgement.recv().await.is_none() {
            println!("ws_disconnected_acknowledgement receiver closed abruptly")
        }

        println!("disconnected successfully")
    }

    /// Return websocket disconnected state to webserver.
    pub async fn is_disconnected(&self) -> bool {
        *self.is_ws_disconnected.read().await
    }

    pub fn wait_for_shutdown(&self) {
        self.waitgroup.wait();
    }

    /// Clear queue, error commands channels and close websocket connection normally.
    /// Shutdown broadcasts a disconnect command to websocket continuosly and waits for waitgroup block to be
    /// closed before exiting.
    pub async fn shutdown(&mut self) {
        if *self.is_ws_disconnected.read().await {
            info!("Websocket already disconnected. Closing connection.");

            return;
        };

        info!("Shutting down websocket.");

        self.disconnect().await;

        info!("Websocket disconnected.");

        self.waitgroup.done();
    }
}
