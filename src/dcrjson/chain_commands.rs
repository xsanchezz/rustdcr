//! Chain Commands.
//! Contains all chain [non-wallet] commands to RPC server. To import this features
//! ChainCommand trait needs to be imported.
use {
    super::rpc_types,
    crate::{dcrjson::RpcJsonError, rpcclient::client::Client},
    log::{debug, trace, warn},
    tokio::sync::mpsc,
    tokio_tungstenite::tungstenite::Message,
};

// ToDo: Currently, async functions are not allowed in traits.
// Move all functions to a traits so as to hide methods.
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
                on_notify_block(msg, block_connected_callback, block_disconnected_callback);
            }

            trace!("Closing notify blocks notification handler.")
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
        if on_new_ticket_callback.is_none() {
            return Err(RpcJsonError::UnregisteredNotification(
                "Notify new tickets".into(),
            ));
        }

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
                on_winning_tickets(msg, on_new_ticket_callback);
            }

            trace!("Closing notify blocks notification handler.")
        });

        todo!()
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

/// Add support for chain commands.
pub trait ChainCommand {
    // fn add_node(&self) -> future_types::AddNodeFuture {
    //     // make some calls
    //     todo!()
    // }

    // fn get_added_node_info(&self) {}

    // fn create_raw_ssr_tx(&self) {}

    // fn create_raw_ss_tx(&self) {}

    // fn create_raw_transaction(&self) {}

    // fn debug_level(&self) {}

    // fn decode_raw_transaction(&self) {}

    // fn estimate_smart_fee(&self) {}

    // fn estimate_stake_diff(&self) {}

    // fn exist_address(&self) {}

    // fn exist_addresses(&self) {}

    // fn exists_expired_tickets(&self) {}

    // fn exists_live_ticket(&self) {}

    // fn exists_live_tickets(&self) {}

    // fn exists_mempool_txs(&self) {}

    // fn exists_missed_tickets(&self) {}

    // fn get_best_block(&self) {}

    // fn get_current_net(&self) {}

    // fn get_headers(&self) {}

    // fn get_stake_difficulty(&self) {}

    // fn get_stake_version_info(&self) {}

    // fn get_stake_versions(&self) {}

    // fn get_ticket_pool_value(&self) {}

    // fn get_vote_info(&self) {}

    // fn live_tickets(&self) {}

    // fn missed_tickets(&self) {}

    // fn session(&self) {}

    // fn ticket_fee_info(&self) {}

    // fn ticket_vwap(&self) {}

    // fn tx_fee_info(&self) {}

    // fn version(&self) {}
}

impl ChainCommand for Client {}

fn on_notify_block(
    msg: tokio_tungstenite::tungstenite::Message,
    block_connected_callback: Option<fn(block_header: Vec<u8>, transactions: Vec<Vec<u8>>)>,
    block_disconnected_callback: Option<fn(block_header: Vec<u8>)>,
) {
    trace!("Received block notification.");

    let msg = msg.into_data();

    match serde_json::from_slice(&msg) {
        Ok(val) => {
            let result: rpc_types::JsonResponse = val;

            let params = match result.params.as_array() {
                Some(params) => params,

                None => {
                    warn!("Invalid params type for notify blocks, expected an array.");
                    return;
                }
            };

            match result.method.as_str() {
                Some(method) => match method {
                    rpc_types::BLOCK_CONNECTED_NOTIFICATION_METHOD => {
                        on_block_connected(params, block_connected_callback)
                    }

                    rpc_types::BLOCK_DISCONNECTED_NOTIFICATION_METHOD => {
                        on_block_disconnected(params, block_disconnected_callback)
                    }

                    _ => {
                        warn!("Server sent an unsupported method type for notify blocks notifications.");
                        return;
                    }
                },

                None => {
                    warn!("Invalid method type for notify blocks expected a string.");
                    return;
                }
            }
        }

        Err(e) => {
            warn!(
                "Error marshalling server result on notify blocks, error: {}, {:?}.",
                e,
                std::str::from_utf8(&msg),
            );

            return;
        }
    };
}

fn on_block_connected(
    params: &Vec<serde_json::Value>,
    on_block_connected: Option<fn(block_header: Vec<u8>, transactions: Vec<Vec<u8>>)>,
) {
    let callback = match on_block_connected {
        Some(callback) => callback,

        None => {
            debug!("On block connected notifier not registered by client.");
            return;
        }
    };

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

    callback(block_header, transactions);
}

fn on_block_disconnected(
    params: &Vec<serde_json::Value>,
    on_block_disconnected: Option<fn(block_header: Vec<u8>)>,
) {
    let callback = match on_block_disconnected {
        Some(callback) => callback,

        None => {
            debug!("On block disconnected notifier not registered by client.");
            return;
        }
    };

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

    callback(block_header);
}

fn on_winning_tickets(
    msg: Message,
    new_tickets_callback: Option<
        fn(
            height: i64,
            stake_diff: i64,
            tickets: Vec<&[u8; crate::chaincfg::chainhash::constants::HASH_SIZE]>,
        ),
    >,
) {
    trace!("Received winning tickets notification.");

    let msg = msg.into_data();

    match serde_json::from_slice(&msg) {
        Ok(val) => {
            let result: rpc_types::JsonResponse = val;

            let params = match result.params.as_array() {
                Some(params) => params,

                None => {
                    warn!("Invalid params type for notify blocks, expected an array.");
                    return;
                }
            };

            match result.method.as_str() {
                Some(method) => match method {
                    rpc_types::NEW_TICKETS_NOTIFICATION_METHOD => {
                        on_new_tickets(params, new_tickets_callback)
                    }

                    _ => {
                        warn!("Server sent an unsupported method type for notify blocks notifications.");
                        return;
                    }
                },

                None => {
                    warn!("Invalid method type for notify blocks expected a string.");
                    return;
                }
            }
        }

        Err(e) => {
            warn!(
                "Error marshalling server result on winning ticket notification, error: {}, {:?}.",
                e,
                std::str::from_utf8(&msg),
            );

            return;
        }
    }
}

fn on_new_tickets(
    params: &Vec<serde_json::Value>,
    new_tickets_callback: Option<
        fn(
            height: i64,
            stake_diff: i64,
            tickets: Vec<&[u8; crate::chaincfg::chainhash::constants::HASH_SIZE]>,
        ),
    >,
) {
    let callback = match new_tickets_callback {
        Some(callback) => callback,

        None => {
            debug!("On block connected notifier not registered by client.");
            return;
        }
    };

    if params.len() != 4 {
        warn!("Server sent wrong number of parameters on new tickets notification handler");
        return;
    }
}
