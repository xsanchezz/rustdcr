//! RPC Client.
//! Contains all client methods to connect to RPC server.

use {
    super::{connection, constants, infrastructure, notify, RpcClientError},
    crate::{dcrjson::chain_command_result::JsonResponse, helper::waitgroup},
    async_std::sync::{Arc, Mutex, RwLock},
    futures::stream::SplitStream,
    log::{info, warn},
    std::{
        collections::{HashMap, VecDeque},
        sync::atomic::{AtomicU64, Ordering},
    },
    tokio::{net::TcpStream, sync::mpsc},
    tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream},
};

/// TLS or TCP Websocket connection connection.
pub type Websocket = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Creates a new RPC client based on the provided connection configuration
/// details.  The notification handlers parameter may be None if you are not
/// interested in receiving notifications and will be ignored if the
/// configuration is set to run in HTTP POST mode.
pub async fn new(
    config: connection::ConnConfig,
    notif_handler: notify::NotificationHandlers,
) -> Result<Client, RpcClientError> {
    let websocket_channel = mpsc::channel(constants::SEND_BUFFER_SIZE);
    let http_channel = mpsc::channel(constants::SEND_BUFFER_SIZE);

    let disconnect_ws_channel = mpsc::channel(1);
    let ws_disconnect_acknowledgement = mpsc::channel(1);

    let mut client = Client {
        id: AtomicU64::new(1),
        configuration: Arc::new(RwLock::new(config)),
        disconnect_ws: disconnect_ws_channel.0,

        is_ws_disconnected: Arc::new(RwLock::new(true)),
        notification_handler: Arc::new(notif_handler),
        notification_state: Arc::new(RwLock::new(HashMap::new())),
        receiver_channel_id_mapper: Arc::new(Mutex::new(HashMap::new())),
        requests_queue_container: Arc::new(Mutex::new(VecDeque::new())),

        ws_user_command: websocket_channel.0,
        http_user_command: http_channel.0,

        ws_disconnected_acknowledgement: ws_disconnect_acknowledgement.1,
        waitgroup: waitgroup::new(),
    };

    let config = client.configuration.clone();
    let mut config = config.write().await;

    client.waitgroup.add(1);

    if !config.disable_connect_on_new && !config.http_post_mode {
        info!("Establishing websocket connection");

        match config.ws_split_stream().await {
            Ok(ws) => {
                client
                    .ws_handler(
                        websocket_channel.1,
                        disconnect_ws_channel.1,
                        ws_disconnect_acknowledgement.0,
                        ws,
                    )
                    .await;

                *client.is_ws_disconnected.write().await = false;
            }

            Err(e) => return Err(e),
        };
    } else if config.http_post_mode {
        let http_mode_future = match create_http_client(&config) {
            Ok(e) => {
                infrastructure::handle_post_methods(e, client.configuration.clone(), http_channel.1)
            }

            Err(e) => return Err(e),
        };

        tokio::spawn(http_mode_future);
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
    pub(crate) id: AtomicU64,

    /// A websocket channel that tunnels converted users messages to websocket write middleman to be consumed by websocket writer.
    pub(crate) ws_user_command: mpsc::Sender<infrastructure::Command>,

    /// An http channel sender that sends clients message to a http writer middleman to be consumed by http client.
    pub(crate) http_user_command: mpsc::Sender<infrastructure::Command>,

    /// A channel that calls for disconnection of websocket connection.
    disconnect_ws: mpsc::Sender<()>,

    /// A channel that acknowledges websocket disconnection.
    ws_disconnected_acknowledgement: mpsc::Receiver<()>,

    /// Holds the connection configuration associated with the client.
    pub(crate) configuration: Arc<RwLock<connection::ConnConfig>>,

    /// Contains all notification callback functions. It is protected by a mutex lock.
    /// To update notification handlers, you need to call an helper method. ToDo create an helper method.
    pub(crate) notification_handler: Arc<notify::NotificationHandlers>,

    /// Used to track the current state of successfully registered notifications so the state can be automatically
    // re-established on reconnect.
    /// On notification registration, message sent to the RPC server is copied and stored. This is so that on reconnection
    /// same message can be sent to the server and server can reply to recently registered command channel which calls the callback
    /// function.
    pub(crate) notification_state: Arc<RwLock<HashMap<String, u64>>>,

    /// Stores all requests to be be sent to the RPC server.
    requests_queue_container: Arc<Mutex<VecDeque<Vec<u8>>>>,

    /// Maps request ID to receiver channel.
    /// Messages received from rpc server are mapped with ID stored.
    pub(crate) receiver_channel_id_mapper: Arc<Mutex<HashMap<u64, mpsc::Sender<JsonResponse>>>>,

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
    async fn ws_handler(
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

        let notification_handler = mpsc::channel(1);

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
            notification_handler.0,
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
            .notification_handler
            .on_client_connected
            .unwrap_or(|| {});

        let reconnect_handler = infrastructure::ws_reconnect_handler(
            self.configuration.clone(),
            self.is_ws_disconnected.clone(),
            signal_ws_reconnect.1,
            new_ws_reader.0,
            new_ws_writer.0,
            self.notification_state.clone(),
            on_client_connected,
        );

        let notification_handler = infrastructure::handle_notification(
            notification_handler.1,
            self.notification_handler.clone(),
        );

        // Separately spawn asynchronous thread for each instances.
        tokio::spawn(websocket_out);
        tokio::spawn(websocket_in);
        tokio::spawn(rcvd_msg_handler);
        tokio::spawn(ws_write_middleman);
        tokio::spawn(reconnect_handler);
        tokio::spawn(notification_handler);

        on_client_connected();

        self.waitgroup.done();
    }

    /// Returns the next id to be used when sending a JSON-RPC message. This ID allows
    /// responses to be associated with particular requests per the JSON-RPC specification.
    /// Typically the consumer of the client does not need to call this function, however,
    /// if a custom request is being created and used this function should be used to ensure the ID
    /// is unique amongst all requests being made.
    pub(crate) fn next_id(&self) -> u64 {
        self.id.fetch_add(1, Ordering::SeqCst)
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
    pub async fn connect(&mut self) -> Result<(), RpcClientError> {
        if !*self.is_ws_disconnected.read().await {
            return Err(RpcClientError::WebsocketAlreadyConnected);
        }

        let mut config = self.configuration.write().await;
        if config.http_post_mode {
            return Err(RpcClientError::ClientNotConnected);
        }

        let user_command_channel = mpsc::channel(1);
        let disconnect_ws_channel = mpsc::channel(1);
        let ws_disconnect_acknowledgement = mpsc::channel(1);

        self.ws_user_command = user_command_channel.0;
        self.disconnect_ws = disconnect_ws_channel.0;
        self.ws_disconnected_acknowledgement = ws_disconnect_acknowledgement.1;

        let ws = match config.ws_split_stream().await {
            Ok(ws) => ws,

            Err(e) => return Err(e),
        };
        drop(config);

        // Change websocket disconnected state.
        let mut is_ws_disconnected = self.is_ws_disconnected.write().await;
        *is_ws_disconnected = false;
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

    /// Allows creating custom RPC command and sends command to server returning a receiving
    /// channel that receives results returned by server.
    pub async fn custom_command(
        &mut self,
        method: &str,
        params: &[serde_json::Value],
    ) -> Result<(u64, mpsc::Receiver<JsonResponse>), RpcClientError> {
        let (id, msg) = self.marshal_command(method, params);

        let msg = match msg {
            Ok(cmd) => cmd,

            Err(e) => {
                warn!("error marshalling custom command, error: {}", e);
                return Err(RpcClientError::Marshaller(e));
            }
        };

        let channel = mpsc::channel(1);

        let cmd = super::infrastructure::Command {
            id: id,
            rpc_message: msg,
            user_channel: channel.0,
        };

        let config = self.configuration.read().await;

        let mut server_channel = if config.http_post_mode {
            self.http_user_command.clone()
        } else {
            self.ws_user_command.clone()
        };

        match server_channel.send(cmd).await {
            Ok(_) => return Ok((id, channel.1)),

            Err(e) => {
                warn!("error sending custom command to server, error: {}", e);

                return Err(RpcClientError::RpcDisconnected);
            }
        };
    }

    /// Marshals clients methods and parameters to a valid JSON RPC command also returning command ID for mapping.
    pub(super) fn marshal_command(
        &self,
        method: &str,
        params: &[serde_json::Value],
    ) -> (u64, Result<Vec<u8>, serde_json::Error>) {
        let id = self.next_id();

        let request = crate::dcrjson::chain_command_result::JsonRequest {
            jsonrpc: "1.0",
            id: id,
            method: method,
            params: params,
        };

        return (id, serde_json::to_vec(&request));
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
            warn!("error sending disconnect command to webserver, disconnect_ws closed.");
            return;
        }

        if self.ws_disconnected_acknowledgement.recv().await.is_none() {
            warn!("ws_disconnected_acknowledgement receiver closed abruptly");
            return;
        }

        info!("disconnected successfully")
    }

    async fn unregister_notification_state(&mut self) {
        let mut notification_state = self.notification_state.write().await;

        notification_state.clear();
    }

    /// Return websocket disconnected state to webserver.
    pub async fn is_disconnected(&self) -> bool {
        *self.is_ws_disconnected.read().await
    }

    /// Blocks until the client is disconnected and connection closed.
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

        self.unregister_notification_state().await;

        self.disconnect().await;

        info!("Websocket disconnected.");

        self.waitgroup.done();
    }
}

fn create_http_client(
    config: &super::connection::ConnConfig,
) -> Result<reqwest::Client, RpcClientError> {
    let proxy = match &config.proxy_host {
        Some(proxy) => {
            let proxy = reqwest::Proxy::all(proxy);

            let proxy = match proxy {
                Ok(e) => e,

                Err(e) => {
                    warn!("Error setting up RPC proxy connection, error: {}", e);
                    return Err(RpcClientError::ProxyConnection);
                }
            };

            let proxy = if config.proxy_password.len() > 0 || config.proxy_username.len() > 0 {
                proxy.basic_auth(&config.proxy_username, &config.proxy_password)
            } else {
                proxy
            };

            Some(proxy)
        }

        None => None,
    };

    let mut request_builder = reqwest::Client::builder();
    request_builder = match proxy {
        Some(e) => request_builder.proxy(e),

        None => request_builder,
    };

    request_builder = match reqwest::Certificate::from_pem(config.certificates.as_bytes()) {
        Ok(certificate) => {
            // ToDo: check if host name is an ip before accepting invalid hostname.
            request_builder
                .add_root_certificate(certificate)
                .danger_accept_invalid_certs(true)
        }

        Err(e) => {
            warn!("Error parsing tls certificate, error: {}", e);
            return Err(RpcClientError::HttpTlsCertificate(e));
        }
    };

    let mut headers = reqwest::header::HeaderMap::new();

    let header_value = match reqwest::header::HeaderValue::from_str("application/json") {
        Ok(e) => e,
        Err(e) => {
            warn!(
                "Failed to set header content type in HTTP Post mode, error: {}",
                e
            );
            return Err(RpcClientError::HttpHeader(e));
        }
    };

    headers.append(reqwest::header::CONTENT_TYPE, header_value);

    let request_builder = request_builder.default_headers(headers);

    let http_client = match request_builder.build() {
        Ok(e) => {
            info!("Successful HTTP handshake");
            e
        }

        Err(e) => {
            info!("Error building HTTP handshake, error: {}", e);
            return Err(RpcClientError::HttpHandshake(e));
        }
    };

    return Ok(http_client);
}
