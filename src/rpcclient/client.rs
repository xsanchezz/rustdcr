use crate::{
    helper::waitgroup,
    rpcclient::{connection, notify},
};

use log::warn;

use async_std::sync::{Arc, Mutex, RwLock};

use futures::stream::SplitStream;
use std::{
    collections::{HashMap, VecDeque},
    error,
    sync::atomic::AtomicU64,
};

use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub type Websocket = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

struct _Command {
    id: u64,
    user_channel: mpsc::Sender<Message>,
    rpc_message: Message,
}

/// Creates a new RPC client based on the provided connection configuration
/// details.  The notification handlers parameter may be None if you are not
/// interested in receiving notifications and will be ignored if the
/// configuration is set to run in HTTP POST mode.
pub async fn new(
    _config: connection::ConnConfig,
    _notif_handler: notify::NotificationHandlers,
) -> Result<Client, String> {
    let (ws_rcv_chan_send, ws_rcv_chan_rcv) = mpsc::channel(1);
    let (disconnect_ws_send, disconnect_ws_rcv) = mpsc::channel(1);

    let mut client = Client {
        _id: AtomicU64::new(0),
        _configuration: Arc::new(RwLock::new(_config)),
        _disconnect_ws: disconnect_ws_send,
        _is_ws_disconnected: Arc::new(RwLock::new(true)),
        _notification_handler: Arc::new(_notif_handler),
        _notification_state: Arc::new(RwLock::new(notify::NotificationState::default())),
        _receiver_channel_id_mapper: Arc::new(RwLock::new(HashMap::new())),
        _requests_queue_container: Arc::new(RwLock::new(VecDeque::new())),
        _websocket_receiver_channel: ws_rcv_chan_send,
        waitgroup: waitgroup::new(),
    };

    let config = client._configuration.clone();
    let mut config = config.write().await;
    if config.disable_connect_on_new && !config.http_post_mode {
        match config.ws_split_stream().await {
            Ok(ws) => {
                client
                    ._ws_handler(ws_rcv_chan_rcv, disconnect_ws_rcv, ws)
                    .await;
            }

            Err(e) => return Err(e),
        }
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

    /// A channel that tunnels messages to its receiver which is consumed by the websocket writer.
    _websocket_receiver_channel: mpsc::Sender<Message>,

    /// Holds the connection configuration associated with the client.
    _configuration: Arc<RwLock<connection::ConnConfig>>,

    /// Contains all notification callback functions. It is protected by a mutex lock.
    /// To update notification handlers, you need to call an helper method. ToDo create an helper method.
    _notification_handler: Arc<notify::NotificationHandlers>,

    /// Stores current state of notification handlers so that they can be re-registered on
    /// websocket disconnect.
    _notification_state: Arc<RwLock<notify::NotificationState>>,

    /// Stores all requests to be be sent to the RPC server.
    _requests_queue_container: Arc<RwLock<VecDeque<Message>>>,

    /// Maps request ID to receiver channel.
    /// Messages received from rpc server are mapped with ID stored.
    _receiver_channel_id_mapper: Arc<RwLock<HashMap<u64, mpsc::Receiver<Message>>>>,

    /// Indicates whether the client is disconnected from the server.
    _is_ws_disconnected: Arc<RwLock<bool>>,

    /// A channel that calls for disconnection of websocket connection.
    _disconnect_ws: mpsc::Sender<()>,

    /// Asynchronously blocks.
    waitgroup: waitgroup::WaitGroup,
}

impl Client {
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
        let mut config = self._configuration.write().await;
        if config.http_post_mode {
            return Err("Not websocket".into());
        }

        let is_ws_disconnected = self._is_ws_disconnected.read().await;
        if *is_ws_disconnected == false {
            return Err("Already connected".into());
        }

        let (ws_rcv_chan_send, ws_rcv_chan_rcv) = mpsc::channel(1);
        let (disconnect_ws_send, disconnect_ws_rcv) = mpsc::channel(1);

        self.waitgroup.add(1);

        self._websocket_receiver_channel = ws_rcv_chan_send;
        self._disconnect_ws = disconnect_ws_send;

        let ws = match config.ws_split_stream().await {
            Ok(ws) => ws,

            Err(e) => return Err(e),
        };

        drop(config);
        drop(is_ws_disconnected);

        self._ws_handler(ws_rcv_chan_rcv, disconnect_ws_rcv, ws)
            .await;

        self.waitgroup.done();
        todo!()
    }

    /// Handles websocket connection to server by calling selective function to handle websocket send, write and reconnect.
    pub(crate) async fn _ws_handler(
        &mut self,
        ws_rcv_chan_rcv: mpsc::Receiver<Message>,
        disconnect_ws_cmd_rcv: mpsc::Receiver<()>,
        split_stream: (Websocket, mpsc::Sender<Message>),
    ) {
    }

    /// Handles sending commands to RPC server through websocket. websocket_out is a `non-blocking` command.
    ///
    /// `ws_sender` is a sender channel that sends RPC commands to its receiver channel which forwards it to websocket,
    ///
    /// `ws_sender_new` is a channel that receives new websocket channel sender on websocket reconnect.
    ///
    /// `queue_command` is a `consumer` which receives RPC commands from command queue. On websocket disconnect
    ///
    /// `message_sent_acknowledgement` acknowledges middleman on websocket send failure or success, it also indicates
    /// to middleman to send next client command in queue. It is important to start an acknowledgement on start or error.
    ///
    /// `request_queue_updated` command is received when command queue has been updated and is to be sent to server.
    /// request_queue_updated start up message queue retrieval when a success message_sent_acknowledgement
    /// has sent to middleman and queue has been emptied.
    ///
    /// `disconnect_cmd_rcv` handle websocket closure.
    ///
    /// When an RPC command is sent, an acknowledgement message is broadcasted to a middle man which either sends next rpc command
    /// in queue on success or resends last errored message on error, middle man also acknowledges user on queue update.
    /// If websocket disconnects either through a protocol error or a normal close, `websocket_out` closes and has to be recalled to
    /// function. Ping commands are sent at intervals.
    #[inline]
    async fn _handle_websocket_out(
        mut _ws_sender: mpsc::Sender<Message>,
        mut _ws_sender_new: mpsc::Receiver<mpsc::Sender<Message>>,
        mut _queue_command: mpsc::Receiver<Message>,
        //  mut ws_state_command: mpsc::Receiver<Message>,
        mut _message_sent_acknowledgement: mpsc::Sender<Result<(), Message>>,
        mut _request_queue_updated: mpsc::Receiver<()>,
        _disconnect_cmd_rcv: mpsc::Receiver<()>,
    ) {
        todo!()
    }

    /// Handles tunneling messages sent by RPC server from server to client. handle_websocket_in is non-blocking.
    ///
    /// `rcv_websocket_msg_handler` tunnels received websocket message in an unbuffered channel so as to be processed by
    /// `received_RPC_message_handler`.
    ///
    /// `websocket_read` continuosly reads messages received from server, if message received is `None` indicates
    /// websocket is closed and needs to be reconnected.
    /// `Note:` On `websocket_read close`, `websocket_writer` is also safely closed and websocket requires a `reconnect`
    /// which is done automatically.
    ///
    /// `websocket_read_new` sends a new websocket stream if websocket disconnects and autoconnect is enabled.
    ///
    /// `is_ws_disconnected` indicates if websocket is disconnected.
    ///
    /// `ws_disconnected_acknowledgement` signals on normal websocket closure this normally returns a result when users calls the Disconnect command.
    ///
    /// `signal_ws_reconnect` signals websocket reconnect handler to create a new websocket connection and send new ws stream through receiving
    /// channels.
    ///
    /// Messages received by websocket stream are sent to a received message handler which processes messages.
    /// If websocket disconnects either through a protocol error or a normal close, `handle_websocket_in` calls for a new websocket connection.
    #[inline]
    async fn _handle_websocket_in(
        _rcv_websocket_msg_handler: mpsc::UnboundedSender<Message>,
        _websocket_read: Websocket,
        _websocket_read_new: mpsc::Receiver<Websocket>,
        _is_ws_disconnected: Arc<RwLock<bool>>,
        _ws_disconnected_acknowledgement: mpsc::Sender<()>,
        _signal_ws_reconnect: mpsc::Sender<()>,
    ) {
        todo!()
    }

    /// Handles received messages from RPC server. handle_received_message is non-blocking.
    ///
    /// `received_msg_consumer` consumes message sent by websocket server.
    /// On websocket disconnect websocket is closed and drained messages are returned back to the top of the queue.
    ///
    /// `receiver_channel_ID_mapper` maps client command sender to receiver channel using unique ID.
    ///
    /// Messages received are unmarshalled and ID gotten, ID is mapped to get client command sender channel.
    /// Sender channel is `disconnected` immediately message is sent to client.
    /// If websocket disconnects either through a protocol error or a normal close, `handle_received_message` closes and has to be recalled to
    /// function
    async fn _handle_received_message(
        _received_msg_consumer: mpsc::UnboundedReceiver<Message>,
        _receiver_channel_id_mapper: Arc<RwLock<HashMap<u64, mpsc::Sender<Message>>>>,
    ) {
        todo!()
    }

    /// Middleman between websocket writer/out and database. ws_write_middleman is non-blocking.
    ///
    /// `user_command` sends users RPC command and a channel to update client async command on success
    /// Users RPC response channel is saved to Mapper against its unique ID. Command sender channel is retrieved from DB and
    /// updated when RPC server responds.
    ///
    /// `request_queue_updated` updates websocket writer when queue is updated.
    ///
    /// `message_send_acknowledgement` is an acknowledgement from websocket writer to send next queue.
    ///
    /// `requests_queue_container` is a container that stored queued users requests.
    ///
    /// `receiver_channel_id_mapper` is a mapper that stores command channels against their ID.
    ///
    /// On user rpc request to server, command is converted to a message and a mpsc channel is created that updates the client asynchronous command
    /// on success.
    /// If websocket disconnects either through a protocol error or a normal close, `ws_write_middleman` closes and has to be recalled to
    /// function.
    async fn _ws_write_middleman(
        mut _user_command: mpsc::Receiver<_Command>,
        mut _request_queue_updated: mpsc::Sender<()>,
        mut _message_send_acknowledgement: mpsc::Receiver<Result<(), Message>>,
        _requests_queue_container: Arc<RwLock<VecDeque<Message>>>,
        _receiver_channel_id_mapper: Arc<RwLock<HashMap<u64, mpsc::Sender<Message>>>>,
    ) {
        todo!()
    }

    /// Pings websocket at intervals and signal for reconnection on websocket disconnect.
    ///
    /// `ping_rpc_command` sends ping or close command to websocket server.
    ///
    /// `disconnect_cmd_rcv` is a channel that `receives` disconnect command from client.
    ///
    /// `is_ws_disconnected` stores websocket disconnection state.
    ///
    /// `signal_ws_reconnect` signals a websocket reconnect command to ws_reconnect_handler function
    /// on websocket failure.
    ///
    /// Pings are sent to websocket server at intervals to keep alive websocket connection,
    async fn _ws_state_handler(
        mut _rpc_ping_close_command: mpsc::Sender<Message>,
        _disconnect_cmd_rcv: mpsc::Receiver<()>,
        _is_ws_disconnected: Arc<RwLock<bool>>,
        _signal_ws_reconnect: mpsc::Sender<()>,
    ) {
        todo!()
    }

    /// Reconnects websocket on failure if user specifies Auto Connect as true.
    ///
    /// `config` contains websocket credentials for a reconnection.
    ///
    /// `ws_reconnect_signal` signals handler to initiate a websocket reconnection.
    ///
    /// `websocket_read_new` sends new websocket stream to handler.
    ///
    /// `ws_writer_new` sends new websocket writer to handler.
    ///
    /// On websocket disconnect a new websocket channel is to be created and sent across handler for
    /// a successful reconnection. Reconnection is only called if Auto Connect is off.
    async fn _ws_reconnect_handler(
        config: Arc<Mutex<connection::ConnConfig>>,
        mut ws_reconnect_signal: mpsc::Receiver<()>,
        mut websocket_read_new: mpsc::Sender<Websocket>,
        mut ws_writer_new: mpsc::Sender<mpsc::Sender<Message>>,
    ) {
        while let Some(_) = ws_reconnect_signal.recv().await {
            let mut backoff = std::time::Duration::new(0, 0);

            loop {
                backoff = backoff + crate::rpcclient::constants::ConnectionRetryIntervalSecs;

                let (ws_rcv, ws_writer) = match config.lock().await.ws_split_stream().await {
                    Ok(ws) => ws,

                    Err(e) => {
                        warn!("unable to reconnect websocket, error: {}", e);
                        continue;
                    }
                };

                match websocket_read_new.send(ws_rcv).await {
                    Ok(_) => {} // Fallthrough to ws_writer_new send.

                    // It is assumed websocket channels are closed, so handler is closed.
                    Err(e) => {
                        warn!(
                            "websocket reconnect handler closed on websocket_read, error: {}",
                            e
                        );
                        return;
                    }
                };

                match ws_writer_new.send(ws_writer).await {
                    Ok(_) => {
                        // Wait for next WS reconnect call.
                        std::thread::sleep(backoff);
                        break;
                    }

                    Err(e) => {
                        warn!(
                            "websocket reconnect handler closed on ws_writer send, error: {}",
                            e
                        );
                        return;
                    }
                };
            }
        }
    }

    /// Disconnects RPC server, deletes command queue and errors any pending request by client.
    pub fn disconnect(&self) {
        todo!()
    }

    /// Return websocket disconnected state to webserver.
    pub fn disconnected(&self) {
        todo!()
    }

    pub fn wait_for_shutdown(&self) {
        self.waitgroup.wait();
    }

    /// Clear queue, error commands channels and close websocket connection normally.
    /// Shutdown broadcasts a disconnect command to websocket continuosly and waits for waitgroup block to be
    /// closed before exiting.
    pub fn shutdown(&self) {
        todo!()
    }
}
