use {
    super::{chain_notification, connection::Websocket},
    crate::{
        dcrjson::{commands, result_types::JsonResponse},
        rpcclient::{connection, constants, infrastructure},
    },
    futures_util::{
        stream::{SplitSink, SplitStream, StreamExt},
        SinkExt,
    },
    log::{debug, info, trace, warn},
    std::{
        collections::{HashMap, VecDeque},
        sync::Arc,
    },
    tokio::{
        sync::{mpsc, Mutex, RwLock},
        time,
    },
    tokio_tungstenite::{tungstenite, tungstenite::Error as WSError, tungstenite::Message},
};

/// Contains RPC Json ID, channel used to send RPC result and message to be sent to server.
pub struct Command {
    /// ID to track server to client commands.
    pub id: u64,
    /// Channel to send received message from server.
    pub user_channel: mpsc::Sender<JsonResponse>,
    /// Message to be send to server server.
    pub rpc_message: Vec<u8>,
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
/// ToDo: Add a condvar to signal all functionalities on websocket close.
pub(super) async fn handle_websocket_in(
    send_rcvd_websocket_msg: mpsc::UnboundedSender<Message>,
    mut websocket_read: SplitStream<Websocket>,
    mut websocket_read_new: mpsc::Receiver<SplitStream<Websocket>>,
    signal_ws_reconnect: mpsc::Sender<()>,
) {
    'outer_loop: loop {
        while let Some(message) = websocket_read.next().await {
            match message {
                // Send received message to message handler function.
                Ok(message) => {
                    if let Err(e) = send_rcvd_websocket_msg.send(message) {
                        // On error indicates send_rcvd_websocket_msg channel
                        // is closed which calls for handle_websocket_in exit.
                        warn!("error sending received websocket message to message handler, error: {}.
                         Closing websocket connection", e);
                        return;
                    }
                }

                Err(e) => {
                    match e {
                        WSError::ConnectionClosed | WSError::AlreadyClosed => {
                            info!("websocket already closed.");
                            return;
                        }

                        _ => {
                            warn!("websocket disconnected unexpectedly with error: {}, calling for reconnection", e);
                            break;
                        }
                    };
                }
            }
        }

        info!("reconnecting websocket");

        // Fall through for reconnection.
        if let Err(e) = signal_ws_reconnect.send(()).await {
            warn!(
                "websocket reconnection failed, error: {}. Closing websocket connection.",
                e
            );
            return;
        }

        let ws = match websocket_read_new.recv().await {
            Some(ws) => ws,

            None => {
                // Websocket auto connect is disabled, return.
                warn!("failed to retrieve new websocket reader. Closing websocket connection.");
                break 'outer_loop;
            }
        };

        // Change to new websocket stream and loop for new connection.
        info!("Changing websocket_read channel.");
        websocket_read = ws;
    }

    info!("handle_websocket_in exited")
}

/// Handles received messages from RPC server. handle_received_message is non-blocking.
///
/// `rcvd_msg_consumer` consumes message sent by websocket server. On websocket disconnect, websocket
/// is closed and drained messages are returned back to the top of the queue.
///
/// `ws_disconnected_acknowledgement` sends websocket disconnect acknowledgement to client.
///
/// `receiver_channel_ID_mapper` maps client command sender to receiver channel using unique ID.
///
/// `notification_handler` sends notification messages to their receiving channel.
///
/// Messages received are unmarshalled and ID gotten, ID is mapped to get client command sender channel.
/// Sender channel is `disconnected` immediately message is sent to client.
/// If websocket disconnects either through a protocol error or a normal close, `handle_received_message` closes and has to be recalled to
/// function.
pub(super) async fn handle_received_message(
    mut rcvd_msg_consumer: mpsc::UnboundedReceiver<Message>,
    notification_handler: mpsc::Sender<JsonResponse>,
    ws_disconnected_acknowledgement: mpsc::Sender<()>,
    receiver_channel_id_mapper: Arc<Mutex<HashMap<u64, mpsc::Sender<JsonResponse>>>>,
) {
    while let Some(message) = rcvd_msg_consumer.recv().await {
        let json_content: JsonResponse = match message {
            Message::Binary(m) => match serde_json::from_slice(&m) {
                Ok(m) => m,

                Err(e) => {
                    warn!(
                        "Error unmarshalling binary result, error: {}. \n Message: {:?}",
                        e,
                        std::str::from_utf8(&m)
                    );

                    continue;
                }
            },

            Message::Text(m) => match serde_json::from_str(&m) {
                Ok(m) => m,

                Err(e) => {
                    warn!(
                        "Error unmarshalling string result, error: {}. \n Message: {}",
                        e, m
                    );

                    continue;
                }
            },

            Message::Close(_) => {
                info!("Received close message from server, closing now.");

                match ws_disconnected_acknowledgement.send(()).await {
                    Ok(_) => {
                        info!("websocket connection closed successfully",);
                    }

                    Err(e) => {
                        warn!("error sending websocket disconnect acknowledgement to client, error: {}", e);
                    }
                };

                return;
            }

            Message::Pong(_) => {
                info!("Received pong message from server");
                continue;
            }

            Message::Ping(_) => {
                info!("Received ping message from server");

                continue;
            }
        };

        // Check if message is a notifier or a command.
        let id = if json_content.id.is_null() {
            debug!("Received a notification");

            match notification_handler.send(json_content).await {
                Ok(_) => {
                    trace!("Sent received notification to handler.");
                    continue;
                }

                Err(e) => {
                    warn!(
                        "Error sending notification message to receiver, error: {}",
                        e
                    );
                    continue;
                }
            };
        } else {
            let id = match json_content.id.as_u64() {
                Some(id) => id,

                None => {
                    warn!(
                        "Unsupported ID value type sent by RPC server, ID consist: {:?}",
                        json_content.id.as_str()
                    );
                    continue;
                }
            };

            id
        };

        let mut receiver_channel_id_mapper = receiver_channel_id_mapper.lock().await;

        match receiver_channel_id_mapper.get_mut(&id) {
            Some(val) => {
                match val.send(json_content).await {
                    Ok(_) => {}

                    Err(e) => {
                        warn!(
                            "Client RPC result receiver channel closed abruptly, error: {}. ID is {}",
                            e, id,
                        );
                    }
                };
            }

            None => warn!("Could not retrieve senders request channel from map"),
        };
    }

    info!("handle_received_message exited");
}

/// Middleman between websocket writer/out and database. ws_write_middleman is non-blocking.
///
/// `user_command` receives a `clients RPC command and a sender channel` to update client async command on success
/// Users RPC response channel is saved to Mapper against its unique ID. Command sender channel is retrieved from DB and
/// updated when RPC server responds.
///
/// `request_queue_updated` updates websocket writer when queue is updated.
///
/// `message_sent_acknowledgement` is an acknowledgement from websocket writer to send next queue.
///
/// `requests_queue_container` is a container that stored queued users requests.
///
/// `receiver_channel_id_mapper` is a mapper that stores command result receiver channels against their ID.
///
/// On user rpc request to server, command is converted to a `Command` which consists of command ID user channel and a result channel
/// that updates on success. User channel is save to database against their ID.
/// If websocket disconnects either through a protocol error or a normal close, `ws_write_middleman` closes and has to be recalled to
/// function.
pub(super) async fn ws_write_middleman(
    mut user_command: mpsc::Receiver<Command>,
    request_queue_updated: mpsc::Sender<()>,
    mut message_sent_acknowledgement: mpsc::Receiver<Result<(), Vec<u8>>>,
    send_queue_command: mpsc::Sender<Vec<u8>>,
    requests_queue_container: Arc<Mutex<VecDeque<Vec<u8>>>>,
    receiver_channel_id_mapper: Arc<Mutex<HashMap<u64, mpsc::Sender<JsonResponse>>>>,
) {
    // Check for updates from client for new commands or
    // websocket writer if it is to send next command in queue.
    loop {
        tokio::select! {
            // Receives `commands` by client which are to be sent to server
            // and saves to message queue, on closure of channel, function is closed.
            command = user_command.recv() => {
                match command {
                    Some(command) => {
                        let mut mapper = receiver_channel_id_mapper.lock().await;

                        if mapper.insert(command.id, command.user_channel).is_some() {
                            warn!("channel ID already present in map, ID: {}.", command.id);
                            break;
                        }
                        drop(mapper);

                        // Update queue and then update websocket writer about queue modification.
                        requests_queue_container
                            .lock()
                            .await
                            .push_back(command.rpc_message);

                        // Signal websocket writer that a queue is updated.
                        if let Some(e) = request_queue_updated.send(()).await.err() {
                            warn!("request_queue_updated sending channel closed, error: {}. Closing websocket connection.", e);
                            break;
                        }
                    }

                    None => {
                        warn!("client command receiving channel closed. Closing websocket connection.");
                        break;
                    },
                }
            }

            ack = message_sent_acknowledgement.recv() => {
                match ack {
                    Some(ack) => {
                        match ack {
                            Ok(_) => {
                                match requests_queue_container.lock().await.pop_front() {
                                    Some(message) => {
                                        if send_queue_command.send(message).await.is_err() {
                                            warn!("Error sending message queue to websocket writer");
                                            break;
                                        }
                                    }

                                    None => info!("queue empty. All commands have been sent to server."),
                                };
                            }

                            Err(message) => {
                                // Place back errored message to the top of queue so as to be re-requested by websocket writer.
                                requests_queue_container.lock().await.push_front(message);

                                // Send back queue updated acknowledgement back to websocket writer.
                                if request_queue_updated.send(()).await.is_err() {
                                    warn!("Request queue updated sending channel closed abruptly");
                                    break;
                                }
                            }
                        }
                    }

                    None => {
                        warn!("message sent acknowledgement channel receiver closed");
                        break;
                    }
                }
            }
        }
    }

    info!("ws_write_middleman exited")
}

/// Handles sending commands to RPC server through websocket.
///
/// `ws_sender` is an mpsc channel that forwars RPC commands to websocket,
///
/// `ws_sender_new` is an mpsc channel that receives new websocket channel sender on websocket reconnect.
///
/// `queue_command` is a `consumer` which receives RPC commands from command queue.
///
/// `message_sent_acknowledgement` acknowledges middleman on websocket send failure or success so as to send
/// next message in queue. On message send failure, message is sent back to queue and requested again when websocket is ready.
/// It is important to start an acknowledgement on start or error.
///
/// `request_queue_updated` signal is received when client command queue has been updated and is ready to be sent to server.
/// request_queue_updated starts up message queue retrieval when a success message_sent_acknowledgement has sent to middleman
/// and queue has been emptied.
///
/// `disconnect_cmd_rcv` handle websocket closure on request from client. On disconnected command, Close message is sent to server
/// and websocket is closed when server acknowledges close command.
///
/// When an RPC command is sent, an acknowledgement message is broadcasted to a middle man which either sends next rpc command
/// in queue on success or resends last errored message on error, middle man also acknowledges user on queue update.
/// If websocket disconnects either through a protocol error or a normal close, `websocket_out` closes and has to be recalled to
/// function. Ping commands are sent at intervals.
pub(super) async fn handle_websocket_out(
    mut ws_sender: mpsc::Sender<Message>,
    mut ws_sender_new: mpsc::Receiver<mpsc::Sender<Message>>,
    mut queue_command: mpsc::Receiver<Vec<u8>>,
    message_sent_acknowledgement: mpsc::Sender<Result<(), Vec<u8>>>,
    mut request_queue_updated: mpsc::Receiver<()>,
    mut disconnect_cmd_rcv: mpsc::Receiver<()>,
) {
    let send_ack = |msg_ack: mpsc::Sender<Result<(), Vec<u8>>>| async move {
        match msg_ack.send(Ok(())).await {
            Ok(_) => {}

            Err(e) => warn!("error sending websocket open acknowledgement, error: {}", e),
        };
    };

    // Websocket writer ready to receive next queue message.
    send_ack(message_sent_acknowledgement.clone()).await;

    let mut ping_sender = ws_sender.clone();

    loop {
        tokio::select! {
            disconnect = disconnect_cmd_rcv.recv() => {
                match disconnect {
                    Some(_) => {
                        let close_message = Message::Close(Some(tungstenite::protocol::CloseFrame {
                            code: tungstenite::protocol::frame::coding::CloseCode::Normal,
                            reason: "".into(),
                        }));

                        match ws_sender.send(close_message).await{
                            Ok(_) => {
                                // TODO: We could still leave the WS open but block sending OUT
                                // then only close till all receiving channels are consumed.
                                info!("Websocket close message sent successfully to server");
                            },

                            Err(e) => {
                                warn!(
                                    "Error sending close message to websocket, error: {}",
                                    e
                                );
                            }
                        };
                    }

                    None => {
                        warn!("Websocket disconnect channel receiver closed abruptly");
                    }
                }

                break;
            }

            // A ping command is sent to server if no RPC command is sent within time frame of 5secs.
            // This is to keep alive connection between websocket server and client.
            _ = time::sleep(tokio::time::Duration::from_secs(constants::KEEP_ALIVE)) => {
                debug!("Sending keep alive ping to websocket server");

                match ping_sender.send(Message::Ping(Vec::new())).await {
                    Ok(_) => {
                        continue;
                    },

                    Err(e) => warn!("Error sending ping message, error: {}", e),
                };
            }

            // Request queue was updated, requesting for next message.
            e = request_queue_updated.recv() => {
                match e {
                    Some(_) => send_ack(message_sent_acknowledgement.clone()).await,

                    // Close websocket connection if request queue channel is closed.
                    None => {
                        warn!("request_queue_update receiver channel closed abruptly");
                        break;
                    }
                }
            }

            // Receives new websocket sender channel if there is a reconnect.
            new_ws = ws_sender_new.recv() => {
                match new_ws {
                    Some(new_ws)=>{
                        ping_sender = new_ws.clone();
                        ws_sender = new_ws;

                        info!("Websocket reconnected");
                        continue;
                    }

                    None => {
                        // If ws_sender_new closes, it is assumed auto connect is disabled on websocket failure.
                        // Exiting handle_websocket_out.
                        warn!("New websocket sender channel closed abruptly. Closing connection.");
                        break;
                    }
                }
            }

            msg = queue_command.recv() => {
                match msg {
                    // `ws_sender` isn't the source of truth if indeed a message was
                    // sent to the websocket as we pipe messages to a future_channel sender
                    // so only the futures channel sender should signify message delivery
                    // acknowledgement.
                    // We only handle error if the tokio sender fails to receive the message
                    // thereby not sending any data to the futures_channel sender.
                    Some(msg) => if let Err(e) = ws_sender.send(Message::Binary(msg)).await {
                        match message_sent_acknowledgement.send(Err(e.0.into_data())).await {
                            Ok(_) => continue,

                            Err(e) => {
                                warn!(
                                    "Error sending message sent acknowledgement error to websocket, error: {}. Closing websocket connection.",
                                    e
                                );
                                break;
                            }
                        }
                    },

                    None => {
                        warn!("command queue receiver closed abruptly, closing websocket connection.");
                        break;
                    }
                }
            }
        }
    }
}

// TODO: this is a bugger, we cant be handling channel context every
// time we need to send messages using the writer.
// We use tokio channel to pipe to our future_channel mpsc.
pub(super) async fn get_ws_sink(
    mut sink: mpsc::Receiver<Message>,
    mut ws_sender: SplitSink<Websocket, Message>,
    ack: mpsc::Sender<Result<(), Vec<u8>>>,
) {
    // TODO: add ack
    tokio::spawn(async move {
        while let Some(msg) = sink.recv().await {
            if let Err(e) = ws_sender.send(msg.clone()).await {
                warn!("websocket sender dropped: {}", e);
                // We return the dropped message back to the top
                // of the queue.
                ack.send(Err(msg.into_data())).await.ok();
                return;
            };
        }
    });
}

/// Reconnects websocket on failure if user specifies Auto Connect as true.
///
/// `config` contains websocket credentials for a reconnection.
///
/// `ws_reconnect_signal` receives signal to initiate a websocket reconnection.
///
/// `websocket_read_new` sends new websocket stream to handler.
///
/// `ws_writer_new` sends new websocket writer to handler.
///
/// `notification_state` contains stored registered notification which are registered on reconnection.
///
/// `on_reconnect` is a callback function defined by client that is called on websocket connection. If a
/// callback function is not defined by user, a unit callback is called.
///
/// On websocket disconnect a new websocket channel is to be created and sent across handler for
/// a successful reconnection. Reconnection is only called if Auto Connect is enabled.
#[allow(clippy::too_many_arguments)]
pub(super) async fn ws_reconnect_handler<F>(
    mut conn: impl connection::RPCConn,
    is_ws_disconnected: Arc<RwLock<bool>>,
    mut ws_reconnect_signal: mpsc::Receiver<()>,
    websocket_read_new: mpsc::Sender<SplitStream<Websocket>>,
    ws_writer_new: mpsc::Sender<mpsc::Sender<Message>>,
    notification_state: Arc<RwLock<HashMap<String, u64>>>,
    message_sent_acknowledgement: mpsc::Sender<Result<(), Vec<u8>>>,
    on_reconnect: F,
) where
    F: Fn(),
{
    while ws_reconnect_signal.recv().await.is_some() {
        info!("reconnecting websocket connection.");

        // Check if client disconnected.
        let is_ws_disconnected_clone = is_ws_disconnected.read().await;
        if *is_ws_disconnected_clone {
            info!("Websocket disconnected by client.");
            break;
        }
        drop(is_ws_disconnected_clone);

        let mut backoff = std::time::Duration::new(0, 0);

        // Drop all websocket connection if auto reconnect is disabled or websocket is disconnected.
        if conn.disable_auto_reconnect() {
            info!("Websocket reconnect disabled. Dropping all websocket handler.");

            let mut is_ws_disconnected_clone = is_ws_disconnected.write().await;
            *is_ws_disconnected_clone = true;

            break;
        }

        // Continuosly retry websocket connection.
        loop {
            backoff += crate::rpcclient::constants::CONNECTION_RETRY_INTERVAL_SECS;

            let (ws_rcv, ws_writer) = match conn.ws_split_stream().await {
                Ok(ws) => ws,

                Err(e) => {
                    warn!("unable to reconnect websocket, error: {}. Reconnecting.", e);

                    std::thread::sleep(backoff);
                    continue;
                }
            };

            let (writer, rcvr) = mpsc::channel(1);

            let message_sent_acknowledgement = message_sent_acknowledgement.clone();

            infrastructure::get_ws_sink(rcvr, ws_writer, message_sent_acknowledgement).await;

            // Register registered notifications on reconnection.
            let notification_state_clone = notification_state.read().await;
            for iter in notification_state_clone.clone().into_iter() {
                debug!("Registering {} notification on reconnection.", iter.0);

                let data = format!(
                    "{{ \"jsonrpc\": \"1.0\", \"method\":\"{}\", \"params\":[], \"id\":{} }}",
                    iter.0, iter.1
                );

                trace!(
                    "Registering notification on reconnection, notification: {}",
                    iter.0
                );

                if let Err(e) = writer.send(Message::Text(data)).await {
                    warn!(
                        "Error registering notification on reconnection, error: {}",
                        e
                    );
                }
            }

            trace!("Reconnection websocket message reader");

            if let Err(e) = websocket_read_new.send(ws_rcv).await {
                warn!(
                    "websocket reconnect handler closed on sending new websocket_read channel, error: {}",
                    e
                );
                break;
            }

            trace!("Reconnection websocket message writer");

            if let Err(e) = ws_writer_new.send(writer).await {
                warn!(
                    "websocket reconnect handler closed on sending new ws_writer send, error: {}",
                    e
                );

                break;
            }

            break;
        }

        on_reconnect();
    }

    info!("_ws_reconnect_handler exited")
}

/// Handles all notifications received by websocket.
///
/// `channel_recv` is the receiving channel that receives all channel from `handle_received_message`.
///
/// `notif` contains all registered notification callbacks.
///
/// RPC notifications are sent to handler and are processed accordingly, registered callbacks are called
/// if available.
/// Note: This function requires websocket connection.
pub(super) async fn handle_notification(
    mut channel_recv: mpsc::Receiver<JsonResponse>,
    notif: Arc<super::notify::NotificationHandlers>,
) {
    while let Some(msg) = channel_recv.recv().await {
        info!("Received notification");

        if msg.params.is_empty() {
            warn!("server sent an invalid notification result: {:?}", msg);
            continue;
        }

        match msg.method.as_str() {
            Some(method) => match method {
                commands::NOTIFICATION_METHOD_BLOCK_CONNECTED => match notif.on_block_connected {
                    Some(e) => chain_notification::on_block_connected(&msg.params, e),

                    None => {
                        warn!("On block connected notification callback not registered.");
                        continue;
                    }
                },

                commands::NOTIFICATION_METHOD_BLOCK_DISCONNECTED => {
                    match notif.on_block_disconnected {
                        Some(e) => chain_notification::on_block_disconnected(&msg.params, e),

                        None => {
                            warn!("On block disconnected notification callback not registered.");
                            continue;
                        }
                    }
                }

                commands::NOTIFICATION_METHOD_WORK => match notif.on_work {
                    Some(e) => chain_notification::on_work(&msg.params, e),

                    None => {
                        warn!("On work notification callback not registered.");
                        continue;
                    }
                },

                commands::NOTIFICATION_METHOD_NEW_TICKETS => match notif.on_new_tickets {
                    Some(e) => chain_notification::on_new_tickets(&msg.params, e),

                    None => {
                        warn!("On new tickets notification callback not registered.");
                        continue;
                    }
                },

                commands::NOTIFICATION_METHOD_TX_ACCEPTED => match notif.on_tx_accepted {
                    Some(e) => chain_notification::on_tx_accepted(&msg.params, e),

                    None => {
                        warn!("On transaction accepted notification callback not registered.");
                        continue;
                    }
                },

                commands::NOTIFICATION_METHOD_TX_ACCEPTED_VERBOSE => {
                    match notif.on_tx_accepted_verbose {
                        Some(e) => chain_notification::on_tx_accepted_verbose(&msg.params, e),

                        None => {
                            warn!("On transaction accepted verbose notification callback not registered.");
                            continue;
                        }
                    }
                }

                commands::NOTIFICATION_METHOD_REORGANIZATION => match notif.on_reorganization {
                    Some(e) => chain_notification::on_reorganization(&msg.params, e),

                    None => {
                        warn!("On block reorganization callback not registered.");
                        continue;
                    }
                },

                _ => match notif.on_unknown_notification {
                    Some(e) => {
                        e(method.to_string(), msg);
                    }

                    None => {
                        warn!(
                            "On unknown notification callback not registered. Method: {}",
                            method
                        );
                        continue;
                    }
                },
            },

            None => {
                warn!("Received a nil or unsupported method type on notify blocks.");
                continue;
            }
        }
    }

    trace!("Closing notification handler.");
}
