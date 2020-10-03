//! Chain Notification Commands.
//! Contains all chain [non-wallet] notification commands to RPC server.
use {
    super::RpcClientError,
    crate::{
        chaincfg::chainhash::Hash,
        dcrjson::{
            chain_command_result, future_types::NotificationsFuture, parse_hex_parameters,
            rpc_types,
        },
        rpcclient::client::Client,
    },
    log::{trace, warn},
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
    pub async fn notify_blocks(&mut self) -> Result<NotificationsFuture, RpcClientError> {
        // Check if user is in HTTP post mode.
        let config = self.configuration.read().await;

        if config.http_post_mode {
            return Err(RpcClientError::ClientNotConnected);
        }
        drop(config);

        if self.is_disconnected().await {
            return Err(RpcClientError::RpcDisconnected);
        }

        let (block_connected_callback, block_disconnected_callback) = (
            self.notification_handler.on_block_connected,
            self.notification_handler.on_block_disconnected,
        );

        if block_connected_callback.is_none() && block_disconnected_callback.is_none() {
            return Err(RpcClientError::UnregisteredNotification(
                "Notify blocks".into(),
            ));
        }

        let notif_future = self
            .create_notification(rpc_types::METHOD_NOTIFY_BLOCKS.to_string())
            .await;

        notif_future
    }

    /// NotifyNewTickets registers the client to receive notifications when blocks are connected to the main
    /// chain and new tickets have matured. The notifications are delivered to the notification handlers
    /// associated with the client. Calling this function has no effect if there are no notification handlers
    /// and will result in an error if the client is configured to run in HTTP POST mode.
    ///
    /// The notifications delivered as a result of this call will be via OnNewTickets.
    ///
    /// NOTE: This is a chain extension and requires a websocket connection.
    pub async fn notify_new_tickets(&mut self) -> Result<NotificationsFuture, RpcClientError> {
        let config = self.configuration.read().await;

        if config.http_post_mode {
            return Err(RpcClientError::ClientNotConnected);
        }
        drop(config);

        if self.is_disconnected().await {
            return Err(RpcClientError::RpcDisconnected);
        }

        let on_new_ticket_callback = self.notification_handler.on_new_tickets;

        if on_new_ticket_callback.is_none() {
            return Err(RpcClientError::UnregisteredNotification(
                "Notify new tickets".into(),
            ));
        }

        let notif_future = self
            .create_notification(rpc_types::METHOD_NOTIFY_NEW_TICKETS.to_string())
            .await;

        notif_future
    }

    pub async fn notify_work(&mut self) -> Result<NotificationsFuture, RpcClientError> {
        let config = self.configuration.read().await;

        if config.http_post_mode {
            return Err(RpcClientError::ClientNotConnected);
        }
        drop(config);

        if self.is_disconnected().await {
            return Err(RpcClientError::RpcDisconnected);
        }

        let on_work_callback = self.notification_handler.on_work;

        if on_work_callback.is_none() {
            return Err(RpcClientError::UnregisteredNotification(
                "Notify work".into(),
            ));
        }

        let notif_future = self
            .create_notification(rpc_types::METHOD_NOTIFIY_NEW_WORK.to_string())
            .await;

        notif_future
    }

    async fn create_notification(
        &mut self,
        method: String,
    ) -> Result<NotificationsFuture, RpcClientError> {
        let (id, result_receiver) = match self.custom_command(&method, &[]).await {
            Ok(e) => e,

            Err(e) => return Err(e),
        };

        // Register notification command to active notifications for reconnection.
        let mut notification_state = self.notification_state.write().await;
        notification_state.insert(method, id);

        Ok(NotificationsFuture {
            message: result_receiver,
        })
    }
}

pub(super) fn on_notification(msg: Message) -> Option<chain_command_result::JsonResponse> {
    let msg = msg.into_data();

    match serde_json::from_slice(&msg) {
        Ok(val) => {
            let result: chain_command_result::JsonResponse = val;

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

pub(super) fn on_block_connected(
    params: &Vec<serde_json::Value>,
    on_block_connected: fn(block_header: Vec<u8>, transactions: Vec<Vec<u8>>),
) {
    trace!("Received on block connected notification");

    if params.len() != 2 {
        warn!("Server sent wrong number of parameters on block connected notification handler");
        return;
    }

    let block_header = match parse_hex_parameters(&params[0]) {
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

pub(super) fn on_block_disconnected(
    params: &Vec<serde_json::Value>,
    on_block_disconnected: fn(block_header: Vec<u8>),
) {
    trace!("Received on block disconnected notification");

    if params.len() != 1 {
        warn!("Server sent wrong number of parameters on block disconnected notification handler");
        return;
    }

    let block_header = match parse_hex_parameters(&params[0]) {
        Some(e) => e,

        None => {
            warn!("Error parsing hex value on block disconnection notification");
            return;
        }
    };

    on_block_disconnected(block_header);
}

pub(super) fn on_new_tickets(
    params: &Vec<serde_json::Value>,
    new_tickets_callback: fn(hash: Hash, height: i64, stake_diff: i64, tickets: Vec<Hash>),
) {
    trace!("Received on new ticket notification");

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

pub(super) fn on_work(
    params: &Vec<serde_json::Value>,
    on_work_callback: fn(data: Vec<u8>, target: Vec<u8>, reason: String),
) {
    trace!("Received on work notification");

    if params.len() != 3 {
        warn!("Server sent wrong number of parameters on new work notification handler");
        return;
    }

    let data = match parse_hex_parameters(&params[0]) {
        Some(e) => e,

        None => {
            warn!("Error getting hex DATA on work notification handler.");
            return;
        }
    };

    let target = match parse_hex_parameters(&params[1]) {
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
