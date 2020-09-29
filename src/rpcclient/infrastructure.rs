use {
    crate::rpcclient::{connection, constants},
    async_std::sync::{Arc, Mutex, RwLock},
    futures::stream::{SplitStream, StreamExt},
    log::{debug, info, trace, warn},
    std::collections::{HashMap, VecDeque},
    tokio::{net::TcpStream, sync::mpsc, time},
    tokio_tungstenite::{
        tungstenite, tungstenite::Error as WSError, tungstenite::Message, MaybeTlsStream,
        WebSocketStream,
    },
};

pub type Websocket = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub(crate) struct Command {
    pub id: u64,
    pub user_channel: mpsc::Sender<Message>,
    pub rpc_message: Message,
}

#[derive(serde::Deserialize)]
pub(super) struct JsonID {
    pub id: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub(super) struct JsonNotificationMethod {
    pub(super) method: String,
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
pub(super) async fn handle_websocket_out(
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

            Err(e) => warn!("error sending websocket open acknowledgement, error: {}", e),
        };
    };

    // Websocket writer ready to receive next queue message.
    send_ack(message_sent_acknowledgement.clone()).await;

    let mut delay = time::delay_for(tokio::time::Duration::from_secs(constants::KEEP_ALIVE));
    let mut ping_sender = ws_sender.clone();

    // ToDo: What happens if auto disconnect is disabled????
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
                                // ToDo: Users should still be able to handle incoming messages till a close is acknowledged by server.
                                info!("websocket close message sent successfully to server");
                            },

                            Err(e) => {
                                warn!(
                                    "error sending close message to websocket, error: {}",
                                    e
                                );
                            }
                        };

                      //  return;
                    }

                    None => {
                        warn!("websocket disconnect channel receiver closed abruptly");

                        return;
                    }
                }
            }

            // A ping command is sent to server if no RPC command is sent within time frame of 5secs.
            // This is to keep alive connection between websocket server and client.
            _ = &mut delay => {
                debug!("sending keep alive ping to websocket server");
                delay.reset(time::Instant::now() + time::Duration::from_secs(constants::KEEP_ALIVE));

                match ping_sender.send(Message::Ping(Vec::new())).await {
                    Ok(_) => {
                        continue;
                    },

                    Err(e) => warn!("error sending ping message, error: {}", e),
                };
            }

            e = request_queue_updated.recv() => {
                match e {
                    Some(_) => send_ack(message_sent_acknowledgement.clone()).await,

                    // Close websocket connection if request queue channel is closed.
                    None => {
                        warn!("request_queue_update receiver channel closed abruptly");

                        return;
                    }
                }
            }

            new_ws = ws_sender_new.recv() => {
                match new_ws {
                    Some(new_ws)=>{
                        ping_sender = new_ws.clone();
                        ws_sender = new_ws;

                        info!("websocket reconnected");
                        continue;
                    }

                    None => {
                        // If ws_sender_new closes, it is assumed auto connect is disabled on websocket failure.
                        // Exiting handle_websocket_out.
                        warn!("new websocket sender channel closed abruptly. Closing connection.");

                        return;
                    }
                }
            }

            msg = queue_command.recv() => {
                match msg {
                    Some(msg) => match ws_sender.send(msg).await {
                        // Send message_sent acknowledgement back to server so as to send next queue in VecQueue.
                        Ok(_) => match message_sent_acknowledgement.send(Ok(())).await {
                            Ok(_) => continue,

                            Err(e) => {
                                warn!(
                                    "error sending message sent acknowledgement success to websocket, error: {}. Closing websocket connection.",
                                    e
                                );

                                return;
                            }
                        },

                        // On channel error indicates either a protocol error and auto reconnect disabled or websocket closing normally
                        // command is sent back to queue.
                        Err(e) => match message_sent_acknowledgement.send(Err(e.0)).await {
                            Ok(_) => continue,

                            Err(e) => {
                                warn!(
                                    "error sending message sent acknowledgement error to websocket, error: {}. Closing websocket connection.",
                                    e
                                );

                                return;
                            }
                        },
                    },

                    None => {
                        warn!("command queue receiver closed abruptly, closing websocket connection.");
                        return;
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
/// ToDo: Add a condvar to signal all functionalities on websocket close.
pub(super) async fn handle_websocket_in(
    send_rcvd_websocket_msg: mpsc::UnboundedSender<Message>,
    mut websocket_read: Websocket,
    mut websocket_read_new: mpsc::Receiver<Websocket>,
    mut signal_ws_reconnect: mpsc::Sender<()>,
) {
    'outer_loop: loop {
        while let Some(message) = websocket_read.next().await {
            match message {
                // Send received message to message handler function.
                Ok(message) => match send_rcvd_websocket_msg.send(message) {
                    Ok(_) => {}

                    Err(e) => {
                        // On error indicates send_rcvd_websocket_msg channel is closed which calls for handle_websocket_in exit.
                        warn!("error sending received websocket message to message handler, error: {}. Closing websocket connection", e);

                        return;
                    }
                },

                Err(e) => {
                    match e {
                        // ToDo: It seems server does not report back a close message on websocket close.
                        // Report on DCRD repo.
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
        match signal_ws_reconnect.send(()).await {
            Ok(_) => {}

            // ToDo: treat error here
            Err(_) => {
                warn!("websocket reconnection failed. Closing websocket connection.");

                return;
            }
        };

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
/// `rcvd_msg_consumer` consumes message sent by websocket server.
/// On websocket disconnect websocket is closed and drained messages are returned back to the top of the queue.
///
/// `ws_disconnected_acknowledgement` sends websocket disconnect acknowledgement to client.
///
/// `receiver_channel_ID_mapper` maps client command sender to receiver channel using unique ID.
///
/// `registered_notification_handler` maps registered notification handlers against their receiving channel.
///
/// Messages received are unmarshalled and ID gotten, ID is mapped to get client command sender channel.
/// Sender channel is `disconnected` immediately message is sent to client.
/// If websocket disconnects either through a protocol error or a normal close, `handle_received_message` closes and has to be recalled to
/// function.
pub(super) async fn handle_received_message(
    mut rcvd_msg_consumer: mpsc::UnboundedReceiver<Message>,
    mut ws_disconnected_acknowledgement: mpsc::Sender<()>,
    receiver_channel_id_mapper: Arc<Mutex<HashMap<u64, mpsc::Sender<Message>>>>,
    registered_notification_handler: Arc<RwLock<HashMap<String, u64>>>,
) {
    while let Some(message) = rcvd_msg_consumer.recv().await {
        let json_content: JsonID = match &message {
            Message::Binary(m) => match serde_json::from_slice(m) {
                Ok(m) => m,

                Err(e) => {
                    warn!(
                        "Error unmarshalling binary result, error: {}. \n Message: {:?}",
                        e,
                        std::str::from_utf8(m)
                    );

                    continue;
                }
            },

            Message::Text(m) => match serde_json::from_str(m) {
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

        let message_clone = message.clone();
        let registered_notification_handler = registered_notification_handler.read().await;

        // Check if message is a notifier or a command.
        let id = if json_content.id.is_null() {
            debug!("Received a notification");
            let notification_method: JsonNotificationMethod =
                match serde_json::from_slice(&message_clone.into_data()) {
                    Ok(e) => e,

                    Err(e) => {
                        warn!("Error marshalling notification json, error: {}", e);
                        continue;
                    }
                };

            let id = match registered_notification_handler.get(&notification_method.method) {
                Some(id) => *id,

                None => {
                    warn!("RPC Notification not registered.");
                    continue;
                }
            };

            id
        } else {
            let id = match json_content.id.as_u64() {
                Some(id) => id,

                None => {
                    warn!(
                        "Unsupported ID value type sent by RPC server, ID consist: {:?}",
                        json_content.id.as_str()
                    );

                    return;
                }
            };

            id
        };

        drop(registered_notification_handler);

        let mut receiver_channel_id_mapper = receiver_channel_id_mapper.lock().await;

        match receiver_channel_id_mapper.get_mut(&id) {
            Some(val) => {
                match val.send(message).await {
                    Ok(_) => {}

                    Err(e) => {
                        warn!(
                            "could not client command result back to client, error: {}",
                            e
                        );
                    }
                };
            }

            None => warn!("could not retrieve senders request channel from map"),
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
/// `receiver_channel_id_mapper` is a mapper that stores command channels against their ID.
///
/// On user rpc request to server, command is converted to a `Command` which consists of command ID user channel and a result channel
/// that updates on success. User channel is save to database against their ID.
/// If websocket disconnects either through a protocol error or a normal close, `ws_write_middleman` closes and has to be recalled to
/// function.
pub(super) async fn ws_write_middleman(
    mut user_command: mpsc::Receiver<Command>,
    mut request_queue_updated: mpsc::Sender<()>,
    mut message_sent_acknowledgement: mpsc::Receiver<Result<(), Message>>,
    mut send_queue_command: mpsc::Sender<Message>,
    requests_queue_container: Arc<Mutex<VecDeque<Message>>>,
    receiver_channel_id_mapper: Arc<Mutex<HashMap<u64, mpsc::Sender<Message>>>>,
) {
    // Check for updates from client for new commands or websocket writer if to send next command in queue.
    loop {
        tokio::select! {
            command = user_command.recv() => {
                match command {
                    Some(command) => {
                        let mut mapper = receiver_channel_id_mapper.lock().await;

                        if mapper.insert(command.id, command.user_channel).is_some(){
                            warn!("channel ID already present in map, ID: {}.", command.id);

                            continue;
                        }

                        // Update queue and update websocket writer about update.
                        requests_queue_container
                            .lock()
                            .await
                            .push_back(command.rpc_message);

                        // Signal websocket writer that a queue is updated.
                        if let Some(e) = request_queue_updated.send(()).await.err() {
                            warn!("request_queue_updated sending channel closed, error: {}. Closing websocket connection.", e);

                            return;
                        }
                    }

                    None => {
                        warn!("client command receiving channel closed. Closing websocket connection.");

                        return;
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
                                            panic!("error sending message queue to websocket writer")
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
                                    warn!("request queue updated sending channel closed abruptly");

                                    return;
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
/// `notification_state` contains stored registered notification which are registered on reconnection.
///
/// `on_reconnect` a callback function defined by client that is called on websocket connection. If not
/// callback function is defined by user, a unit callback is called.
///
/// On websocket disconnect a new websocket channel is to be created and sent across handler for
/// a successful reconnection. Reconnection is only called if Auto Connect is enabled.
pub(super) async fn ws_reconnect_handler(
    config: Arc<Mutex<connection::ConnConfig>>,
    is_ws_disconnected: Arc<RwLock<bool>>,
    mut ws_reconnect_signal: mpsc::Receiver<()>,
    mut websocket_read_new: mpsc::Sender<Websocket>,
    mut ws_writer_new: mpsc::Sender<mpsc::Sender<Message>>,
    notification_state: Arc<RwLock<HashMap<String, u64>>>,
    on_reconnect: fn(),
) {
    while let Some(_) = ws_reconnect_signal.recv().await {
        info!("reconnecting websocket connection.");

        // Check if client disconnected.
        let is_ws_disconnected_clone = is_ws_disconnected.read().await;
        if *is_ws_disconnected_clone {
            info!("Websocket disconnected by client.");

            return;
        }
        drop(is_ws_disconnected_clone);

        let mut backoff = std::time::Duration::new(0, 0);

        // Drop all websocket connection if auto reconnect is disabled or websocket is disconnected.
        let mut config_clone = config.lock().await;
        if config_clone.disable_auto_reconnect {
            info!("Websocket reconnect disabled. Dropping all websocket handler.");

            let mut is_ws_disconnected_clone = is_ws_disconnected.write().await;
            *is_ws_disconnected_clone = true;

            return;
        }

        // Continuosly retry websocket connection.
        loop {
            backoff = backoff + crate::rpcclient::constants::CONNECTION_RETRY_INTERVAL_SECS;

            let (ws_rcv, mut ws_writer) = match config_clone.ws_split_stream().await {
                Ok(ws) => ws,

                Err(e) => {
                    warn!("unable to reconnect websocket, error: {}. Reconnecting.", e);

                    std::thread::sleep(backoff);
                    continue;
                }
            };

            // Register registered notifications on reconnection.
            let notification_state_clone = notification_state.read().await;
            for iter in notification_state_clone.clone().into_iter() {
                debug!("Registering {} notification on reconnection.", iter.0);

                let data = format!(
                    "{{ \"jsonrpc\": \"1.0\", \"method\":\"{}\", \"params\":[], \"id\":{} }}",
                    iter.0, iter.1
                );

                match ws_writer.send(Message::Text(data)).await {
                    Ok(_) => trace!(
                        "Registering notification on reconnection, notification: {:?}",
                        iter
                    ),

                    Err(e) => warn!(
                        "Error registering notification on reconnection, error: {}",
                        e
                    ),
                }
            }

            match websocket_read_new.send(ws_rcv).await {
                Ok(_) => {} // Fallthrough to ws_writer_new send.

                // It is assumed websocket channels are closed, so handler is closed.
                Err(e) => {
                    warn!(
                        "websocket reconnect handler closed on sending new websocket_read channel, error: {}",
                        e
                    );

                    return;
                }
            };

            match ws_writer_new.send(ws_writer).await {
                Ok(_) => {}

                Err(e) => {
                    warn!(
                        "websocket reconnect handler closed on sending new ws_writer send, error: {}",
                        e
                    );

                    return;
                }
            };

            break;
        }

        on_reconnect();
    }

    info!("_ws_reconnect_handler exited")
}
