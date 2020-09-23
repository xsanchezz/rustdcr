use crate::{
    helper::waitgroup,
    rpcclient::{connection, notify},
};

use log::{info, warn};

use async_std::sync::{Arc, Mutex, RwLock};

use futures::stream::{SplitStream, StreamExt};
use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::AtomicU64,
};

use tokio::{net::TcpStream, sync::mpsc, time};
use tokio_tungstenite::{
    tungstenite::Error as WSError, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};

pub type Websocket = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// SendBufferSize is the number of elements the websocket send channel
/// can queue before blocking.
const SEND_BUFFER_SIZE: usize = 50;

const KEEP_ALIVE: u64 = 5;

pub(crate) struct _Command {
    pub id: u64,
    pub user_channel: mpsc::Sender<Message>,
    pub rpc_message: Message,
}

/// Creates a new RPC client based on the provided connection configuration
/// details.  The notification handlers parameter may be None if you are not
/// interested in receiving notifications and will be ignored if the
/// configuration is set to run in HTTP POST mode.
pub async fn new(
    _config: connection::ConnConfig,
    _notif_handler: notify::NotificationHandlers,
) -> Result<Client, String> {
    let (ws_rcv_chan_send, ws_rcv_chan_rcv) = mpsc::channel(SEND_BUFFER_SIZE);
    let (disconnect_ws_send, disconnect_ws_rcv) = mpsc::channel(1);
    let ws_disconnect_acknowledgement = mpsc::channel(1);

    let mut client = Client {
        _id: AtomicU64::new(0),
        _configuration: Arc::new(Mutex::new(_config)),
        _disconnect_ws: disconnect_ws_send,
        _is_ws_disconnected: Arc::new(RwLock::new(true)),
        _notification_handler: Arc::new(_notif_handler),
        _notification_state: Arc::new(RwLock::new(notify::NotificationState::default())),
        _receiver_channel_id_mapper: Arc::new(RwLock::new(HashMap::new())),
        _requests_queue_container: Arc::new(RwLock::new(VecDeque::new())),
        _user_command: ws_rcv_chan_send,
        _ws_disconnected_acknowledgement: ws_disconnect_acknowledgement.1,
        waitgroup: waitgroup::new(),
    };

    let config = client._configuration.clone();
    let mut config = config.lock().await;

    if config.disable_connect_on_new && !config.http_post_mode {
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

    /// A channel that tunnels converted users messages to ws_write_middleman to be consumed by websocket writer.
    _user_command: mpsc::Sender<_Command>, // ToDo: not needed

    /// A channel that calls for disconnection of websocket connection.
    _disconnect_ws: mpsc::Sender<()>,

    /// A channel that acknowledges websocket disconnection.
    _ws_disconnected_acknowledgement: mpsc::Receiver<()>,

    /// Holds the connection configuration associated with the client.
    _configuration: Arc<Mutex<connection::ConnConfig>>,

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
    _receiver_channel_id_mapper: Arc<RwLock<HashMap<u64, mpsc::Sender<Message>>>>,

    /// Indicates whether the client is disconnected from the server.
    _is_ws_disconnected: Arc<RwLock<bool>>,

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
        let mut config = self._configuration.lock().await;
        if config.http_post_mode {
            return Err("Not websocket".into());
        }

        let mut is_ws_disconnected = self._is_ws_disconnected.write().await;
        if *is_ws_disconnected == false {
            return Err("Already connected".into());
        }

        let user_command_channel = mpsc::channel(1);
        let disconnect_ws_channel = mpsc::channel(1);
        let ws_disconnect_acknowledgement = mpsc::channel(1);

        self.waitgroup.add(1);

        self._user_command = user_command_channel.0;
        self._disconnect_ws = disconnect_ws_channel.0;
        self._ws_disconnected_acknowledgement = ws_disconnect_acknowledgement.1;

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

        self.waitgroup.done();

        Ok(())
    }

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
    pub(crate) async fn ws_handler(
        &mut self,
        user_command: mpsc::Receiver<_Command>,
        disconnect_ws_cmd_rcv: mpsc::Receiver<()>,
        ws_disconnect_acknowledgement: mpsc::Sender<()>,
        split_stream: (Websocket, mpsc::Sender<Message>),
    ) {
        self.waitgroup.add(1);

        let new_ws_writer = mpsc::channel(1);

        let queue_command = mpsc::channel(1);

        let msg_acknowledgement = mpsc::channel(1);

        let request_queue_update = mpsc::channel(1);

        let websocket_out = Self::_handle_websocket_out(
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

        let websocket_in = Self::handle_websocket_in(
            handle_rcvd_msg.0,
            split_stream.0,
            new_ws_reader.1,
            self._is_ws_disconnected.clone(),
            ws_disconnect_acknowledgement,
            signal_ws_reconnect.0,
        );

        let rcvd_msg_handler = Self::_handle_received_message(
            handle_rcvd_msg.1,
            self._receiver_channel_id_mapper.clone(),
        );

        let ws_write_middleman = Self::_ws_write_middleman(
            user_command,
            request_queue_update.0,
            msg_acknowledgement.1,
            queue_command.0,
            self._requests_queue_container.clone(),
            self._receiver_channel_id_mapper.clone(),
        );

        let reconnect_handler = Self::_ws_reconnect_handler(
            self._configuration.clone(),
            signal_ws_reconnect.1,
            new_ws_reader.0,
            new_ws_writer.0,
        );

        // Separately spawn asynchronous thread for each instances.
        tokio::spawn(websocket_out);
        tokio::spawn(websocket_in);
        tokio::spawn(rcvd_msg_handler);
        tokio::spawn(ws_write_middleman);
        tokio::spawn(reconnect_handler);

        self.waitgroup.done();
    }

    /// Handles sending commands to RPC server through websocket. websocket_out is a `non-blocking` command.
    ///
    /// `ws_sender` is a mpsc sender channel that sends RPC commands to its receiver channel which then forwards it to websocket,
    ///
    /// `ws_sender_new` is a channel that receives new websocket channel sender on websocket reconnect.
    ///
    /// `queue_command` is a `consumer` which receives RPC commands from command queue.
    ///
    /// `message_sent_acknowledgement` acknowledges middleman on websocket send failure or success, it also indicates
    /// to middleman to send next client command in queue. It is important to start an acknowledgement on start or error.
    ///
    /// `request_queue_updated` command is received when command queue has been updated and is to be sent to server.
    /// request_queue_updated start up message queue retrieval when a success message_sent_acknowledgement
    /// has sent to middleman and queue has been emptied.
    ///
    /// `disconnect_cmd_rcv` handle websocket closure on request from client.
    ///
    /// When an RPC command is sent, an acknowledgement message is broadcasted to a middle man which either sends next rpc command
    /// in queue on success or resends last errored message on error, middle man also acknowledges user on queue update.
    /// If websocket disconnects either through a protocol error or a normal close, `websocket_out` closes and has to be recalled to
    /// function. Ping commands are sent at intervals.
    #[inline]
    async fn _handle_websocket_out(
        mut ws_sender: mpsc::Sender<Message>,
        mut ws_sender_new: mpsc::Receiver<mpsc::Sender<Message>>,
        mut queue_command: mpsc::Receiver<Message>,
        mut message_sent_acknowledgement: mpsc::Sender<Result<(), Message>>,
        mut request_queue_updated: mpsc::Receiver<()>,
        mut disconnect_cmd_rcv: mpsc::Receiver<()>,
    ) {
        let send_ack = |mut msg_ack: mpsc::Sender<Result<(), Message>>| async move {
            match msg_ack.send(Ok(())).await {
                Ok(_) => {}

                Err(e) => panic!("error sending websocket open acknowledgement, error: {}", e),
            };
        };

        send_ack(message_sent_acknowledgement.clone()).await;

        let mut delay = time::delay_for(tokio::time::Duration::from_secs(KEEP_ALIVE));
        let mut ping_sender = ws_sender.clone();

        // ToDo: What happens if auto disconnect is disabled????
        loop {
            tokio::select! {
                _ = disconnect_cmd_rcv.recv()=>{
                    match ws_sender.send(Message::Close(None)).await{
                        Ok(_) => break,

                        Err(e) => {
                            panic!(
                                "error sending close message to websocket, error: {}",
                                e
                            );
                        }
                    };
                }

                // A ping command is sent to server if no RPC command is sent within time frame of 5secs.
                // This is to keep alive connection between websocket server and client.
                _ = &mut delay => {
                    match ping_sender.send(Message::Ping(Vec::new())).await {
                        Ok(_) => {
                            delay.reset(time::Instant::now() + time::Duration::from_secs(KEEP_ALIVE));
                            continue;
                        },

                        Err(e) => panic!("error sending ping message, error: {}", e),
                    };
                }

                e = request_queue_updated.recv() => {
                    match e {
                        Some(_) => send_ack(message_sent_acknowledgement.clone()).await,

                        None => {
                            panic!("request_queue_update receiver channel closed")
                        }
                    }
                }

                new_ws = ws_sender_new.recv()=>{
                    match new_ws {
                        Some(new_ws)=>{
                            ping_sender = new_ws.clone();
                            ws_sender = new_ws;
                        }

                        None => {
                            // If ws_sender_new closes, it is assumed auto connect is disabled on websocket failure.
                            // Exiting handle_websocket_out.
                            warn!("websocket disconnected");
                            return;
                        }
                    }
                }

                msg = queue_command.recv()=>{
                    match msg {
                        Some(msg) => match ws_sender.send(msg).await {
                            Ok(_) => match message_sent_acknowledgement.send(Ok(())).await {
                                Ok(_) => continue,

                                Err(e) => {
                                    panic!(
                                        "error sending message sent acknowledgement success to websocket, error: {}",
                                        e
                                    );
                                }
                            },

                            // On error indicates either a protocol error or websocket closing normally, command is sent back to queue
                            // and websocket i
                            Err(e) => match message_sent_acknowledgement.send(Err(e.0)).await {
                                Ok(_) => continue,

                                Err(e) => panic!(
                                    "error sending message sent acknowledgement error to websocket, error: {}",
                                    e
                                ),
                            },
                        },

                        None => {
                            panic!("command queue receiver closed abruptly");
                        }
                    }
                }
            }
        }
    }

    /// Handles tunneling messages sent by RPC server from server to client. handle_websocket_in is non-blocking.
    ///
    /// `send_rcvd_websocket_msg` tunnels received websocket message in an unbuffered channel so as to be processed by
    /// `received_RPC_message_handler`.
    ///
    /// `websocket_read` reads messages received from server, if message received is `None` indicates websocket
    /// is closed and needs to be reconnected.
    /// `Note:` On `websocket_read close`, `websocket_writer` is also safely closed and websocket requires a `reconnect`
    /// which is done automatically.
    ///
    /// `websocket_read_new` is a channel that retrieves a reconnected websocket on websocket disconnect, this is if reconnect is enabled.
    ///
    /// `is_ws_disconnected` indicates if websocket is disconnected.
    ///
    /// `ws_disconnected_acknowledgement` signals on normal websocket closure this normally returns a result when users calls the Disconnect command.
    ///
    /// `signal_ws_reconnect` signals websocket reconnect handler to create a new websocket connection and send new ws stream through receiving
    /// channels.
    ///
    /// Handles messages received from websocket read which are sent to a message handler which processes received messages.
    /// If websocket disconnects either through a protocol error or a normal close, `handle_websocket_in` calls for a new websocket connection.
    #[inline]
    async fn handle_websocket_in(
        send_rcvd_websocket_msg: mpsc::UnboundedSender<Message>,
        mut websocket_read: Websocket,
        mut websocket_read_new: mpsc::Receiver<Websocket>,
        is_ws_disconnected: Arc<RwLock<bool>>,
        mut ws_disconnected_acknowledgement: mpsc::Sender<()>,
        mut signal_ws_reconnect: mpsc::Sender<()>,
    ) {
        loop {
            while let Some(message) = websocket_read.next().await {
                match message {
                    Ok(message) => match send_rcvd_websocket_msg.send(message) {
                        Ok(_) => {}

                        Err(e) => {
                            panic!("error sending received websocket message to message handler, error: {}", e);
                        }
                    },

                    Err(e) => {
                        match e {
                            WSError::ConnectionClosed | WSError::AlreadyClosed => {
                                info!("websocket closed.");

                                let mut change_connection_state = is_ws_disconnected.write().await;
                                *change_connection_state = true;

                                match ws_disconnected_acknowledgement.send(()).await {
                                    Ok(_) => {
                                        return;
                                    }

                                    Err(e) => {
                                        panic!("error sending websocket disconnect acknowledgement to client, error: {}", e)
                                    }
                                };
                            }

                            _ => {
                                warn!("websocket disconnected unexpectedly with error: {}", e);

                                let mut change_connection_state = is_ws_disconnected.write().await;
                                *change_connection_state = true;

                                match signal_ws_reconnect.send(()).await {
                                    Ok(_) => {
                                        break;
                                    }

                                    Err(e) => panic!(
                                        "error signalling websocket reconnection, error: {}",
                                        e
                                    ),
                                };
                            }
                        };
                    }
                }
            }

            let ws = match websocket_read_new.recv().await {
                Some(ws) => ws,

                None => {
                    // Websocket auto connect is disabled, return.
                    return;
                }
            };

            // Change to new websocket stream and loop for new connection.
            websocket_read = ws;
        }
    }

    /// Handles received messages from RPC server. handle_received_message is non-blocking.
    ///
    /// `rcvd_msg_consumer` consumes message sent by websocket server.
    /// On websocket disconnect websocket is closed and drained messages are returned back to the top of the queue.
    ///
    /// `receiver_channel_ID_mapper` maps client command sender to receiver channel using unique ID.
    ///
    /// Messages received are unmarshalled and ID gotten, ID is mapped to get client command sender channel.
    /// Sender channel is `disconnected` immediately message is sent to client.
    /// If websocket disconnects either through a protocol error or a normal close, `handle_received_message` closes and has to be recalled to
    /// function.
    async fn _handle_received_message(
        _rcvd_msg_consumer: mpsc::UnboundedReceiver<Message>,
        _receiver_channel_id_mapper: Arc<RwLock<HashMap<u64, mpsc::Sender<Message>>>>,
    ) {
        todo!()
    }

    /// Middleman between websocket writer/out and database. ws_write_middleman is non-blocking.
    ///
    /// `user_command` receives a `clients RPC command and a sender channel` to update client async command on success
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
    /// On user rpc request to server, command is converted to a `Command` which consists of command ID user channel and an a result channel
    /// that updates on success. User channel is save to database against their ID.
    /// If websocket disconnects either through a protocol error or a normal close, `ws_write_middleman` closes and has to be recalled to
    /// function.
    async fn _ws_write_middleman(
        mut _user_command: mpsc::Receiver<_Command>,
        mut _request_queue_updated: mpsc::Sender<()>,
        mut _message_send_acknowledgement: mpsc::Receiver<Result<(), Message>>,
        _send_queue_command: mpsc::Sender<Message>,
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
    /// a successful reconnection. Reconnection is only called if Auto Connect is enabled.
    async fn _ws_reconnect_handler(
        config: Arc<Mutex<connection::ConnConfig>>,
        mut ws_reconnect_signal: mpsc::Receiver<()>,
        mut websocket_read_new: mpsc::Sender<Websocket>,
        mut ws_writer_new: mpsc::Sender<mpsc::Sender<Message>>,
    ) {
        let mut config_clone = config.lock().await;

        if config_clone.disable_auto_reconnect {
            drop(ws_reconnect_signal);
            drop(websocket_read_new);
            drop(ws_writer_new);

            return;
        }

        while let Some(_) = ws_reconnect_signal.recv().await {
            let mut backoff = std::time::Duration::new(0, 0);

            loop {
                backoff = backoff + crate::rpcclient::constants::CONNECTION_RETRY_INTERVAL_SECS;

                let (ws_rcv, ws_writer) = match config_clone.ws_split_stream().await {
                    Ok(ws) => ws,

                    Err(e) => {
                        warn!("unable to reconnect websocket, error: {}", e);
                        std::thread::sleep(backoff);
                        continue;
                    }
                };

                match websocket_read_new.send(ws_rcv).await {
                    Ok(_) => {} // Fallthrough to ws_writer_new send.

                    // It is assumed websocket channels are closed, so handler is closed.
                    Err(e) => {
                        panic!(
                            "websocket reconnect handler closed on sending new websocket_read channel, error: {}",
                            e
                        );
                    }
                };

                match ws_writer_new.send(ws_writer).await {
                    Ok(_) => {
                        // Break loop and wait for next websocket reconnect command.
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
