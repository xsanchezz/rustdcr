#[forbid(missing_docs)]
use crate::{dcrjson::RpcJsonError, rpcclient::client::Client};

use super::{future_types, rpc_types};

use tokio::sync::mpsc;

use log::{debug, trace, warn};

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
        // Check if notification handler has already been registered;
        let notification_state = self.notification_state.write().await;
        let notif_id = match notification_state.get(rpc_types::BLOCK_CONNECTED_METHOD_NAME) {
            Some(notif_id) => Some(*notif_id),

            None => None,
        };

        drop(notification_state);

        // Close sender notification handler channel if already registered.
        match notif_id {
            Some(id) => {
                let mut mapper = self.receiver_channel_id_mapper.lock().await;
                mapper.remove(&id);
            }

            None => {}
        }

        let config = self.configuration.lock().await;
        if config.http_post_mode {
            return Err(RpcJsonError::WebsocketDisabled);
        }
        drop(config);

        let (block_connected_callback, block_disconnected_callback) = (
            self.notification_handler.on_block_connected,
            self.notification_handler.on_block_disconnected,
        );

        if block_connected_callback.is_none() && block_disconnected_callback.is_none() {
            return Err(RpcJsonError::UnregisteredNotification(
                "Notify blocks".into(),
            ));
        }

        let (id, cmd) = self.marshal_command(rpc_types::NOTIFY_BLOCKS_METHOD_NAME, &[]);

        let msg = match cmd {
            Ok(cmd) => cmd,

            Err(e) => return Err(RpcJsonError::Marshaller(e)),
        };

        let (channel_send, mut channel_recv) = mpsc::channel(1);

        let cmd = crate::rpcclient::Command {
            id: id,
            rpc_message: tokio_tungstenite::tungstenite::Message::Binary(msg.clone()),
            user_channel: channel_send,
        };

        match self.user_command.send(cmd).await {
            Ok(_) => {}

            Err(e) => {
                warn!(
                    "Error sending notify blocks command to RPC server, error: {}.",
                    e
                );

                return Err(RpcJsonError::WebsocketClosed);
            }
        }

        let mut notification_state = self.notification_state.write().await;
        notification_state.insert(rpc_types::BLOCK_CONNECTED_METHOD_NAME.to_string(), id);
        notification_state.insert(rpc_types::BLOCK_DISCONNECTED_METHOD_NAME.to_string(), id);
        drop(notification_state);

        tokio::spawn(async move {
            while let Some(msg) = channel_recv.recv().await {
                trace!("Received block notification");

                let msg = msg.into_data();

                match serde_json::from_slice(&msg) {
                    Ok(val) => {
                        let result: rpc_types::JsonResponse = val;

                        match result.method.as_str() {
                            Some(method) => match method {
                                rpc_types::BLOCK_CONNECTED_METHOD_NAME => {
                                    match result.params.as_array() {
                                        Some(params) => {
                                            on_block_connected(params, block_connected_callback)
                                        }

                                        None => {
                                            warn!("Invalid params type for notify blocks, expected an array.");
                                            continue;
                                        }
                                    }
                                }

                                rpc_types::BLOCK_DISCONNECTED_METHOD_NAME => {}

                                _ => {
                                    warn!("Server sent an unsupported method type for notify blocks notifications.");
                                    continue;
                                }
                            },

                            None => {
                                warn!("Invalid method type for notify blocks expected a string.");
                                continue;
                            }
                        }
                    }

                    Err(e) => {
                        warn!(
                            "Error marshalling server result on notify blocks, error: {}, {:?}.",
                            e,
                            std::str::from_utf8(&msg),
                        );

                        continue;
                    }
                };
            }

            trace!("Closing notify blocks notification handler.")
        });

        Ok(())
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

pub trait Extension {
    fn add_node(&self) -> future_types::AddNodeFuture {
        // make some calls
        todo!()
    }

    fn get_added_node_info(&self) {}

    fn create_raw_ssr_tx(&self) {}

    fn create_raw_ss_tx(&self) {}

    fn create_raw_transaction(&self) {}

    fn debug_level(&self) {}

    fn decode_raw_transaction(&self) {}

    fn estimate_smart_fee(&self) {}

    fn estimate_stake_diff(&self) {}

    fn exist_address(&self) {}

    fn exist_addresses(&self) {}

    fn exists_expired_tickets(&self) {}

    fn exists_live_ticket(&self) {}

    fn exists_live_tickets(&self) {}

    fn exists_mempool_txs(&self) {}

    fn exists_missed_tickets(&self) {}

    fn get_best_block(&self) {}

    fn get_current_net(&self) {}

    fn get_headers(&self) {}

    fn get_stake_difficulty(&self) {}

    fn get_stake_version_info(&self) {}

    fn get_stake_versions(&self) {}

    fn get_ticket_pool_value(&self) {}

    fn get_vote_info(&self) {}

    fn live_tickets(&self) {}

    fn missed_tickets(&self) {}

    fn session(&self) {}

    fn ticket_fee_info(&self) {}

    fn ticket_vwap(&self) {}

    fn tx_fee_info(&self) {}

    fn version(&self) {}
}

impl Extension for Client {}

fn on_block_connected(
    params: &Vec<serde_json::Value>,
    on_block_connected: Option<fn(block_header: Vec<u8>, transactions: Vec<Vec<u8>>)>,
) {
    let callback = match on_block_connected {
        Some(callback) => callback,

        None => {
            debug!("On block notifier not registered by client.");
            return;
        }
    };

    if params.len() != 2 {
        warn!("Server sent wrong number of parameters on block connected notification handler");
        return;
    }

    let block_header = match super::parse_hex_parameters(&params[0]) {
        Some(e) => e,

        None => Vec::new(),
    };

    if params[1].is_null() {}

    let hex_transactions: Vec<String> = match serde_json::from_value(params[1].clone()) {
        Ok(e) => e,

        Err(e) => {
            warn!(
                "Error marshalling on block transaction hex transaction values, error: {}",
                e
            );

            Vec::new()
        }
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
