//! Chain Notification Commands.
//! Contains all chain [non-wallet] notification commands to RPC server.
use {
    super::rpc_types,
    crate::{chaincfg::chainhash::Hash, dcrjson::RpcJsonError, rpcclient::client::Client},
    log::{debug, trace, warn},
    tokio::sync::mpsc,
    tokio_tungstenite::tungstenite::Message,
};

// ToDo: Currently, async functions are not allowed in traits.
// Move all functions to a traits so as to hide methods also, we need one helper on_notifier function
// to reduce code duplication.
impl Client {
    /// Registers the client to receive notifications when blocks are
    /// connected and disconnected from the main chain.  The notifications are
    /// delivered to the notification handlers associated with the client.  Calling
    /// this function has no effect if there are no notification handlers and will
    /// result in an error if the client is configured to run in HTTP POST mode.
    ///
    /// The notifications delivered as a result of this call will be via one of
    /// OnBlockConnected or OnBlockDisconnected.
    ///
    /// NOTE: This is a non-wallet extension and requires a websocket connection.
    pub async fn notify_blocks(&mut self) -> Result<(), RpcJsonError> {
        // Check if user is in HTTP post mode.
        let config = self.configuration.read().await;

        if config.http_post_mode {
            return Err(RpcJsonError::WebsocketDisabled);
        }
        drop(config);

        self.close_notification_channel_if_exists(rpc_types::BLOCK_CONNECTED_NOTIFICATION_METHOD)
            .await;
        self.close_notification_channel_if_exists(
            rpc_types::BLOCK_DISCONNECTED_NOTIFICATION_METHOD,
        )
        .await;

        let (block_connected_callback, block_disconnected_callback) = (
            self.notification_handler.on_block_connected,
            self.notification_handler.on_block_disconnected,
        );

        if block_connected_callback.is_none() && block_disconnected_callback.is_none() {
            return Err(RpcJsonError::UnregisteredNotification(
                "Notify blocks".into(),
            ));
        }

        let (mut channel_recv, id) =
            match self.create_command(rpc_types::NOTIFY_BLOCKS_METHOD).await {
                Ok(e) => e,

                Err(e) => return Err(e),
            };

        // Register notification command to active notifications for reconnection.
        let mut notification_state = self.notification_state.write().await;
        notification_state.insert(rpc_types::NOTIFY_BLOCKS_METHOD.to_string(), id);
        drop(notification_state);

        // Register notification method against their receiving channel.
        let mut registered_notif = self.registered_notification_handler.write().await;
        registered_notif.insert(
            rpc_types::BLOCK_DISCONNECTED_NOTIFICATION_METHOD.to_string(),
            id,
        );
        registered_notif.insert(
            rpc_types::BLOCK_CONNECTED_NOTIFICATION_METHOD.to_string(),
            id,
        );
        drop(registered_notif);

        tokio::spawn(async move {
            while let Some(msg) = channel_recv.recv().await {
                let result = on_notification(msg);

                let result = match result {
                    Some(result) => result,

                    None => {
                        warn!("Received a invalid or null response from RPC server.");
                        continue;
                    }
                };

                match result.method.as_str() {
                    Some(method) => {
                        match method {
                            rpc_types::BLOCK_CONNECTED_NOTIFICATION_METHOD => {
                                match block_connected_callback {
                                    Some(e) => on_block_connected(&result.params, e),

                                    None => {
                                        warn!("On block connected notification callback not registered.");
                                        continue;
                                    }
                                }
                            }

                            rpc_types::BLOCK_DISCONNECTED_NOTIFICATION_METHOD => {
                                match block_disconnected_callback {
                                    Some(e) => on_block_disconnected(&result.params, e),

                                    None => {
                                        warn!("On block disconnected notification callback not registered.");
                                        continue;
                                    }
                                }
                            }

                            _ => {
                                warn!("Server sent an unsupported method type for notify blocks notifications.");
                                continue;
                            }
                        }
                    }

                    None => {
                        warn!("Received a nil or unsupported method type on notify blocks.");
                        continue;
                    }
                }
            }

            trace!("Closing notify blocks notification handler.");
        });

        Ok(())
    }

    /// NotifyNewTickets registers the client to receive notifications when blocks are connected to the main
    /// chain and new tickets have matured. The notifications are delivered to the notification handlers
    /// associated with the client. Calling this function has no effect if there are no notification handlers
    /// and will result in an error if the client is configured to run in HTTP POST mode.
    ///
    /// The notifications delivered as a result of this call will be via OnNewTickets.
    ///
    /// NOTE: This is a chain extension and requires a websocket connection.
    pub async fn notify_new_tickets(&mut self) -> Result<(), RpcJsonError> {
        let config = self.configuration.read().await;

        if config.http_post_mode {
            return Err(RpcJsonError::WebsocketDisabled);
        }
        drop(config);

        let on_new_ticket_callback = self.notification_handler.on_new_tickets;

        let on_new_ticket_callback = match on_new_ticket_callback {
            Some(e) => e,

            None => {
                return Err(RpcJsonError::UnregisteredNotification(
                    "Notify new tickets".into(),
                ))
            }
        };

        self.close_notification_channel_if_exists(rpc_types::NOTIFY_NEW_TICKETS_METHOD)
            .await;

        let (mut channel_recv, id) = match self
            .create_command(rpc_types::NOTIFY_NEW_TICKETS_METHOD)
            .await
        {
            Ok(e) => e,

            Err(e) => return Err(e),
        };

        // Register notification command to active notifications for reconnection.
        let mut notification_state = self.notification_state.write().await;
        notification_state.insert(rpc_types::NOTIFY_NEW_TICKETS_METHOD.to_string(), id);
        drop(notification_state);

        // Register notification method against their receiving channel.
        let mut registered_notif = self.registered_notification_handler.write().await;
        registered_notif.insert(rpc_types::NEW_TICKETS_NOTIFICATION_METHOD.to_string(), id);
        drop(registered_notif);

        tokio::spawn(async move {
            while let Some(msg) = channel_recv.recv().await {
                let result = on_notification(msg);

                let result = match result {
                    Some(result) => result,

                    None => {
                        warn!("Received a invalid or null response from RPC server.");
                        continue;
                    }
                };

                match result.method.as_str() {
                    Some(method) => match method {
                        rpc_types::NEW_TICKETS_NOTIFICATION_METHOD => {
                            on_new_tickets(&result.params, on_new_ticket_callback)
                        }

                        _ => {
                            warn!("Server sent an unsupported method type for notify blocks notifications.");
                            continue;
                        }
                    },

                    None => {
                        warn!("Received a nil or unsupported method type on notify blocks.");
                        continue;
                    }
                }
            }

            trace!("Closing notify blocks notification handler.")
        });

        Ok(())
    }

    pub async fn notify_work(&mut self) -> Result<(), RpcJsonError> {
        let config = self.configuration.read().await;

        if config.http_post_mode {
            return Err(RpcJsonError::WebsocketDisabled);
        }
        drop(config);

        let on_work_callback = self.notification_handler.on_work;

        let on_work_callback = match on_work_callback {
            Some(e) => e,

            None => return Err(RpcJsonError::UnregisteredNotification("Notify work".into())),
        };

        self.close_notification_channel_if_exists(rpc_types::NOTIFIY_NEW_WORK_METHOD)
            .await;

        let (mut channel_recv, id) = match self
            .create_command(rpc_types::NOTIFIY_NEW_WORK_METHOD)
            .await
        {
            Ok(e) => e,

            Err(e) => return Err(e),
        };

        // Save notification method as an active state so as to be re-registered on reconnection.
        let mut notification_state = self.notification_state.write().await;
        notification_state.insert(rpc_types::NOTIFIY_NEW_WORK_METHOD.to_string(), id);
        drop(notification_state);

        // Register notification method against their receiving channel.
        let mut registered_notif = self.registered_notification_handler.write().await;
        registered_notif.insert(rpc_types::WORK_NOTIFICATION_METHOD.to_string(), id);
        drop(registered_notif);

        tokio::spawn(async move {
            while let Some(msg) = channel_recv.recv().await {
                let result = on_notification(msg);

                let result = match result {
                    Some(result) => result,

                    None => {
                        warn!("Received a invalid or null response from RPC server.");
                        continue;
                    }
                };

                match result.method.as_str() {
                    Some(method) => match method {
                        rpc_types::WORK_NOTIFICATION_METHOD => {
                            on_work(&result.params, on_work_callback)
                        }

                        _ => {
                            warn!("Server sent an unsupported method type for notify blocks notifications.");
                            continue;
                        }
                    },

                    None => {
                        warn!("Received a nil or unsupported method type on notify blocks.");
                        continue;
                    }
                }
            }

            trace!("Closing notify blocks notification handler.")
        });

        Ok(())
    }

    /// Closes former notification channel if a client decides to register a new notification.
    async fn close_notification_channel_if_exists(&self, rpc_type: &str) {
        // Check if notification handler has already been registered;
        let notification_state = self.notification_state.read().await;

        let notif_channel_id = match notification_state.get(rpc_type) {
            Some(notif_id) => *notif_id,

            None => return,
        };

        // Close sender notification handler channel if already registered.
        let mut mapper = self.receiver_channel_id_mapper.lock().await;
        mapper.remove(&notif_channel_id);
    }

    async fn create_command(
        &mut self,
        method: &str,
    ) -> Result<(mpsc::Receiver<Message>, u64), RpcJsonError> {
        let (id, cmd) = self.marshal_command(method, &[]);

        let msg = match cmd {
            Ok(cmd) => cmd,

            Err(e) => {
                warn!("Error marshalling JSON command, error: {}", e);
                return Err(RpcJsonError::Marshaller(e));
            }
        };

        let (channel_send, channel_recv) = mpsc::channel(1);

        let cmd = crate::rpcclient::Command {
            id: id,
            rpc_message: Message::Binary(msg),
            user_channel: channel_send,
        };

        match self.user_command.send(cmd).await {
            Ok(_) => trace!("Registered notification notify block command."),

            Err(e) => {
                warn!(
                    "Error sending notify blocks command to RPC server, error: {}.",
                    e
                );

                return Err(RpcJsonError::WebsocketClosed);
            }
        }

        Ok((channel_recv, id))
    }

    /// Marshals clients methods and parameters to a valid JSON RPC command also returning command ID for mapping.
    pub fn marshal_command(
        &self,
        method: &str,
        params: &[serde_json::Value],
    ) -> (u64, Result<Vec<u8>, serde_json::Error>) {
        let id = self.next_id();

        let request = rpc_types::JsonRequest {
            jsonrpc: "1.0",
            id: id,
            method: method,
            params: params,
        };

        return (id, serde_json::to_vec(&request));
    }
}

fn on_notification(msg: Message) -> Option<rpc_types::JsonResponse> {
    trace!("Received winning tickets notification.");

    let msg = msg.into_data();

    match serde_json::from_slice(&msg) {
        Ok(val) => {
            let result: rpc_types::JsonResponse = val;

            if !result.params.is_empty() {
                return Some(result);
            }

            return None;
        }

        Err(e) => {
            warn!(
                "Error marshalling server result on notification, error: {}, {:?}.",
                e,
                std::str::from_utf8(&msg),
            );

            return None;
        }
    }
}

fn on_block_connected(
    params: &Vec<serde_json::Value>,
    on_block_connected: fn(block_header: Vec<u8>, transactions: Vec<Vec<u8>>),
) {
    if params.len() != 2 {
        warn!("Server sent wrong number of parameters on block connected notification handler");
        return;
    }

    let block_header = match super::parse_hex_parameters(&params[0]) {
        Some(e) => e,

        None => {
            warn!("Error parsing hex value on block connected notification.");
            return;
        }
    };

    let hex_transactions = if params[1].is_null() {
        Vec::new()
    } else {
        let hex_transactions: Vec<String> = match serde_json::from_value(params[1].clone()) {
            Ok(e) => e,

            Err(e) => {
                warn!(
                    "Error marshalling on block transaction hex transaction values, error: {}",
                    e
                );

                return;
            }
        };

        hex_transactions
    };

    let mut transactions = Vec::new();

    for hex_transaction in hex_transactions {
        match ring::test::from_hex(hex_transaction.as_str()) {
            Ok(v) => transactions.push(v),

            Err(e) => {
                warn!(
                    "Error getting hex value transaction on block connected notifier, error: {}",
                    e
                );
                return;
            }
        };
    }

    on_block_connected(block_header, transactions);
}

fn on_block_disconnected(
    params: &Vec<serde_json::Value>,
    on_block_disconnected: fn(block_header: Vec<u8>),
) {
    if params.len() != 1 {
        warn!("Server sent wrong number of parameters on block disconnected notification handler");
        return;
    }

    let block_header = match super::parse_hex_parameters(&params[0]) {
        Some(e) => e,

        None => {
            warn!("Error parsing hex value on block disconnection notification");
            return;
        }
    };

    on_block_disconnected(block_header);
}

fn on_new_tickets(
    params: &Vec<serde_json::Value>,
    new_tickets_callback: fn(hash: Hash, height: i64, stake_diff: i64, tickets: Vec<Hash>),
) {
    if params.len() != 4 {
        warn!("Server sent wrong number of parameters on new tickets notification handler");
        return;
    }

    let block_sha_str: String = match serde_json::from_value(params[0].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error unmarshalling block SHA params in on new tickets notification, error: {}",
                e
            );
            return;
        }
    };

    let sha_hash = match crate::chaincfg::chainhash::Hash::new_from_str(&block_sha_str) {
        Ok(e) => e,

        Err(e) => {
            warn!("Error converting sha string to chain hash in on new ticket notification, error: {}", e);
            return;
        }
    };

    let block_height: i64 = match serde_json::from_value(params[1].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error marshalling block height in on new tickets notification, error: {}",
                e
            );
            return;
        }
    };

    let stake_diff: i64 = match serde_json::from_value(params[2].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error marshalling stake diff in on new tickets notification, error: {}",
                e
            );
            return;
        }
    };

    let tickets_str: Vec<String> = match serde_json::from_value(params[3].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error marshalling tickets in on new tickets notification, error: {}",
                e
            );
            return;
        }
    };

    let mut tickets: Vec<Hash> = Vec::with_capacity(tickets_str.len());

    for ticket_str in tickets_str.iter() {
        match Hash::new_from_str(ticket_str) {
            Ok(e) => {
                tickets.push(e);
            }

            Err(e) => {
                warn!("Error converting tickets string to hash, error: {}", e);
                return;
            }
        }
    }

    new_tickets_callback(sha_hash, block_height, stake_diff, tickets)
}

fn on_work(
    params: &Vec<serde_json::Value>,
    on_work_callback: fn(data: Vec<u8>, target: Vec<u8>, reason: String),
) {
    if params.len() != 3 {
        warn!("Server sent wrong number of parameters on new work notification handler");
        return;
    }

    let data = match super::parse_hex_parameters(&params[0]) {
        Some(e) => e,

        None => {
            warn!("Error getting hex DATA on work notification handler.");
            return;
        }
    };

    let target = match super::parse_hex_parameters(&params[1]) {
        Some(e) => e,

        None => {
            warn!("Error getting hex TARGET on work notification handler.");
            return;
        }
    };

    let reason: String = match serde_json::from_value(params[2].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error getting on work REASON parmeter on work notification handler, error: {}.",
                e
            );
            return;
        }
    };

    on_work_callback(data, target, reason);
}
