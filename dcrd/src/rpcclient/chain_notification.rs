//! Chain Notification Commands.
//! Contains all chain non-wallet notification commands to RPC server.
use {
    super::RpcClientError,
    crate::{
        chaincfg::chainhash::Hash,
        dcrjson::{future_types::NotificationsFuture, parse_hex_parameters, rpc_types},
        rpcclient::client::Client,
    },
    log::{trace, warn},
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
            .create_notification(rpc_types::METHOD_NOTIFY_BLOCKS.to_string(), &[])
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
            .create_notification(rpc_types::METHOD_NOTIFY_NEW_TICKETS.to_string(), &[])
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
            .create_notification(rpc_types::METHOD_NOTIFIY_NEW_WORK.to_string(), &[])
            .await;

        notif_future
    }

    pub async fn notify_new_transactions(
        &mut self,
        verbose: bool,
    ) -> Result<NotificationsFuture, RpcClientError> {
        let config = self.configuration.read().await;

        if config.http_post_mode {
            return Err(RpcClientError::ClientNotConnected);
        }
        drop(config);

        if self.is_disconnected().await {
            return Err(RpcClientError::RpcDisconnected);
        }

        let callbacks = (
            self.notification_handler.on_tx_accepted,
            self.notification_handler.on_tx_accepted_verbose,
        );

        if callbacks.0.is_none() && callbacks.1.is_none() {
            return Err(RpcClientError::UnregisteredNotification(
                "Notify new transactions".into(),
            ));
        }

        let notif_future = self
            .create_notification(
                rpc_types::METHOD_NEW_TX.to_string(),
                &[serde_json::Value::Bool(verbose)],
            )
            .await;

        notif_future
    }

    /// Registers the client to receive notifications when blocks are connected
    /// to the main chain and stake difficulty is updated. The notifications are delivered to
    /// the notification handlers associated with the client. Calling this function has no effect
    /// if there are no notification handlers and will result in an error if the client is
    /// configured to run in HTTP POST mode.
    pub async fn notify_stake_difficulty(&mut self) -> Result<NotificationsFuture, RpcClientError> {
        let config = self.configuration.read().await;

        if config.http_post_mode {
            return Err(RpcClientError::ClientNotConnected);
        }
        drop(config);

        if self.is_disconnected().await {
            return Err(RpcClientError::RpcDisconnected);
        }

        if self.notification_handler.on_stake_difficulty.is_none() {
            return Err(RpcClientError::UnregisteredNotification(
                "Notify stake difficulty".into(),
            ));
        }

        let notif_future = self
            .create_notification(rpc_types::METHOD_STAKE_DIFFICULTY.to_string(), &[])
            .await;

        notif_future
    }

    async fn create_notification(
        &mut self,
        method: String,
        params: &[serde_json::Value],
    ) -> Result<NotificationsFuture, RpcClientError> {
        let (id, result_receiver) = match self.send_custom_command(&method, params).await {
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

pub(super) fn on_reorganization(
    params: &Vec<serde_json::Value>,
    on_reorganization_callback: fn(
        old_hash: Hash,
        old_height: i32,
        new_hash: Hash,
        new_height: i32,
    ),
) {
    trace!("Received on reorganization notification");

    if params.len() != 4 {
        warn!("Server sent wrong number of parameters on reorganization notification handler");
        return;
    }

    let old_hash = match crate::dcrjson::marshal_to_hash(params[0].clone()) {
        Some(e) => e,

        None => {
            warn!("Error marshalling old_hash params in on reorganization notifiaction.");
            return;
        }
    };

    let old_height: i32 = match serde_json::from_value(params[1].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error unmarshalling old height params in reorganization notification, error: {}",
                e
            );
            return;
        }
    };

    let new_hash = match crate::dcrjson::marshal_to_hash(params[2].clone()) {
        Some(e) => e,

        None => {
            warn!("Error marshalling new_hash params in on reorganization notifiaction.");
            return;
        }
    };

    let new_height: i32 = match serde_json::from_value(params[3].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error unmarshalling new height params in reorganization notification, error: {}",
                e
            );
            return;
        }
    };

    on_reorganization_callback(old_hash, old_height, new_hash, new_height)
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

    let hash = match crate::dcrjson::marshal_to_hash(params[0].clone()) {
        Some(e) => e,

        None => {
            warn!("Error marshalling to hash in on new ticket notification.");
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

    new_tickets_callback(hash, block_height, stake_diff, tickets)
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

pub(super) fn on_tx_accepted(
    params: &Vec<serde_json::Value>,
    on_tx_callback: fn(hash: Hash, amount: crate::dcrutil::amount::Amount),
) {
    trace!("Received transaction accepted notification");

    if params.len() != 2 {
        warn!(
            "Server sent wrong number of parameters on transaction accepted notification handler"
        );
        return;
    }

    let hash = match crate::dcrjson::marshal_to_hash(params[0].clone()) {
        Some(e) => e,

        None => {
            warn!("Error marshalling value to hash in on transaction accepted");
            return;
        }
    };

    let amount_unit: f64 = match serde_json::from_value(params[1].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error marshalling amount unit in on transaction accepted notification, error: {}",
                e
            );
            return;
        }
    };

    let amount = match crate::dcrutil::amount::new(amount_unit) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error converting marshalled number to Amount type, error: {}",
                e
            );
            return;
        }
    };

    on_tx_callback(hash, amount)
}

pub(super) fn on_tx_accepted_verbose(
    params: &Vec<serde_json::Value>,
    on_tx_verbose_callback: fn(tx_details: crate::dcrjson::chain_command_result::TxRawResult),
) {
    trace!("Received transaction accepted verbose notification");

    if params.len() != 1 {
        warn!(
            "Server sent wrong number of parameters on transaction accepted verbsose notification handler"
        );
        return;
    }

    let tx_details: crate::dcrjson::chain_command_result::TxRawResult =
        match serde_json::from_value(params[0].clone()) {
            Ok(e) => e,

            Err(e) => {
                warn!(
                    "Error marshalling transaction accepted verbose notification, error: {}",
                    e
                );
                return;
            }
        };

    on_tx_verbose_callback(tx_details);
}

pub(super) fn on_stake_difficulty(
    params: &Vec<serde_json::Value>,
    on_stake_difficulty_callback: fn(hash: Hash, height: i64, stake_diff: i64),
) {
    trace!("Received stake difficulty notification");

    if params.len() != 3 {
        warn!("Server sent wrong number of parameters on stake difficulty notification handler");
        return;
    }

    let hash = match crate::dcrjson::marshal_to_hash(params[0].clone()) {
        Some(e) => e,

        None => {
            warn!("Error marshalling hash on stake difficulty notification");
            return;
        }
    };

    let block_height: i64 = match serde_json::from_value(params[1].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error unmarshalling block height params in on stake difficulty notification, error: {}",
                e
            );
            return;
        }
    };

    let stake_diff: i64 = match serde_json::from_value(params[2].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error unmarshalling stake diff params in on stake difficulty notification, error: {}",
                e
            );
            return;
        }
    };

    on_stake_difficulty_callback(hash, block_height, stake_diff)
}
