//! Chain Notification Commands.
//! Contains all chain non-wallet notification commands to RPC server.

use super::connection::RPCConn;

use {
    super::{check_config, error::RpcClientError, future_type::NotificationsFuture},
    crate::{
        chaincfg::chainhash::Hash,
        dcrjson::{commands, parse_hex_parameters, result_types},
        rpcclient::client::Client,
    },
    log::{trace, warn},
};

macro_rules! notification_generator {
    ($doc: tt, $name: ident, $return_type: ty, $command: expr, $param: expr, ($($callback_name: tt),*), ($($fn_params:ident : $fn_type: ty),*)) => {
        #[doc = $doc]
        pub async fn $name(&mut self, $($fn_params : $fn_type),*) -> Result<$return_type, RpcClientError> {
            check_config!(self);
            callback_check!(self, $command, ($($callback_name),*));
            create_notif_future!(self, $command, $param)
        }
    };

    ($doc: tt, $name: ident, $return_type: ty, $command: expr, $param: expr, or($($callback_name: tt),*), ($($fn_params:ident : $fn_type: ty),*)) => {
        #[doc = $doc]
        pub async fn $name(&mut self, $($fn_params : $fn_type),*) -> Result<$return_type, RpcClientError> {
            check_config!(self);
            callback_check!(self, $command, or ($($callback_name),*));
            create_notif_future!(self, $command, $param)
        }
    };

    ($doc: tt, $name: ident, $return_type: ty, $command: expr, $param: expr, and($($callback_name: tt),*), ($($fn_params:ident : $fn_type: ty),*)) => {
        #[doc = $doc]
        pub async fn $name(&mut self, $($fn_params : $fn_type),*) -> Result<$return_type, RpcClientError> {
            check_config!(self);
            callback_check!(self, $command, and ($($callback_name),*));
            create_notif_future!(self, $command, $param)
        }
    };
}

macro_rules! callback_check {
    ($self: ident, $name: expr, and($($callback_name: tt), *)) => {
        $(
            let $callback_name = $self.notification_handler.$callback_name;
        )*
        if $(
            $callback_name.is_none()
        ) && * {
            return Err(RpcClientError::UnregisteredNotification(
                $name.to_string(),
            ));
        }
    };

    ($self: ident, $name: expr, ($($callback_name: tt), *)) => {
        $(
            let $callback_name = $self.notification_handler.$callback_name;
        )*
       if $(
            $callback_name.is_none()
        ) || * {
            return Err(RpcClientError::UnregisteredNotification(
                $name.to_string(),
            ));
        }
    };
}

macro_rules! create_notif_future {
    ($self: ident, $command: expr, $param: expr) => {{
        let notif_future = $self.create_notification($command, $param).await;

        notif_future
    }};
}

impl<C: 'static + RPCConn> Client<C> {
    notification_generator!(
        "notify_blocks registers the client to receive notifications when blocks are
        connected and disconnected from the main chain.  The notifications are
        delivered to the notification handlers associated with the client.  Calling
        this function has no effect if there are no notification handlers and will
        result in an error if the client is configured to run in HTTP POST mode.

        The notifications delivered as a result of this call will be via one of
        OnBlockConnected or OnBlockDisconnected.

        NOTE: This is a non-wallet extension and requires a websocket connection.",
        notify_blocks,
        NotificationsFuture,
        commands::METHOD_NOTIFY_BLOCKS.to_string(),
        &[],
        (on_block_connected, on_block_disconnected),
        ()
    );

    notification_generator!(
        "notify_new_tickets registers the client to receive notifications when blocks are connected to the main
        chain and new tickets have matured. The notifications are delivered to the notification handlers
        associated with the client. Calling this function has no effect if there are no notification handlers
        and will result in an error if the client is configured to run in HTTP POST mode.
        The notifications delivered as a result of this call will be via OnNewTickets.
        NOTE: This is a chain extension and requires a websocket connection.)",
        notify_new_tickets,
        NotificationsFuture,
        commands::METHOD_NOTIFY_NEW_TICKETS.to_string(),
        &[],
        (on_new_tickets),
        ()
    );

    notification_generator!(
        "notify_work registers the client to receive notifications when a new block
        template has been generated.

        The notifications delivered as a result of this call will be via on_work.

        NOTE: This is a dcrd extension and requires a websocket connection",
        notify_work,
        NotificationsFuture,
        commands::METHOD_NOTIFIY_NEW_WORK.to_string(),
        &[],
        (on_work),
        ()
    );

    notification_generator!(
        "notify_new_transactions registers the client to receive notifications every
        time a new transaction is accepted to the memory pool.  The notifications are
        delivered to the notification handlers associated with the client.  Calling
        this function has no effect if there are no notification handlers and will
        result in an error if the client is configured to run in HTTP POST mode.
        The notifications delivered as a result of this call will be via one of
        on_tx_accepted (when verbose is false) or on_tx_accepted_verbose (when verbose is
        true).
        
        NOTE: This is a dcrd extension and requires a websocket connection.",
        notify_new_transactions,
        NotificationsFuture,
        commands::METHOD_NEW_TX.to_string(),
        &[serde_json::json!(verbose)],
        and(on_tx_accepted, on_tx_accepted_verbose),
        (verbose: bool)
    );

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
    params: &[serde_json::Value],
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
    params: &[serde_json::Value],
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
    params: &[serde_json::Value],
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
    params: &[serde_json::Value],
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
    params: &[serde_json::Value],
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
    params: &[serde_json::Value],
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
    params: &[serde_json::Value],
    on_tx_verbose_callback: fn(tx_details: result_types::TxRawResult),
) {
    trace!("Received transaction accepted verbose notification");

    if params.len() != 1 {
        warn!(
            "Server sent wrong number of parameters on transaction accepted verbsose notification handler"
        );
        return;
    }

    let tx_details: result_types::TxRawResult = match serde_json::from_value(params[0].clone()) {
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
