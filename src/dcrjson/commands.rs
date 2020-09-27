#[deny(missing_docs)]
use crate::rpcclient::client::Client;

use super::{future_types, jsonrpc};

use tokio::sync::mpsc;

use log::{debug, warn};

// {"jsonrpc":"1.0","result":103789,"error":null,"id":2}

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
    pub async fn notify_blocks(&mut self) -> Result<(), String> {
        const BLOCK_CONNECTED_METHOD_NAME: &str = "blockconnected";
        const BLOCK_DISCONNECTED_METHOD_NAME: &str = "blockdisconnected";

        const METHOD_NAME: &str = "blockdisconnected";

        // Check if notification handler has already been registered;
        let notification_state = self._notification_state.write().await;
        let notif_id = match notification_state.get(BLOCK_CONNECTED_METHOD_NAME) {
            Some(notif_id) => Some(*notif_id),

            None => None,
        };

        drop(notification_state);

        // Close notification handler channel if already registered.
        match notif_id {
            Some(id) => {
                let mut mapper = self.receiver_channel_id_mapper.lock().await;
                mapper.remove(&id);
            }

            None => {}
        }

        let config = self.configuration.lock().await;
        if config.http_post_mode {
            return Err("Websocket required to use this feature.".into());
        }
        drop(config);

        if self._notification_handler.on_block_connected.is_none()
            && self._notification_handler.on_block_disconnected.is_none()
        {
            return Err("Blocks connected or disconnected callback functions are required to be defined to use this function.".into());
        }

        let (id, cmd) = self.marshal_command(METHOD_NAME, &[]);

        let msg = match cmd {
            Ok(cmd) => cmd,

            Err(e) => {
                warn!("Error marshalling notify blocks command, error: {}.", e);
                return Err("Command marshal error".into());
            }
        };

        let (channel_send, mut channel_recv) = mpsc::channel(1);

        let cmd = crate::rpcclient::Command {
            id: id,
            rpc_message: tungstenite::Message::Binary(msg.clone()),
            user_channel: channel_send,
        };

        match self.user_command.send(cmd).await {
            Ok(_) => {}

            Err(e) => {
                warn!(
                    "Error sending notify blocks command to RPC server, error: {}.",
                    e
                );

                return Err("Error sending notify command blocks to RPC.".into());
            }
        }

        let mut notification_state = self._notification_state.write().await;
        notification_state.insert(BLOCK_CONNECTED_METHOD_NAME.to_string(), id);
        notification_state.insert(BLOCK_DISCONNECTED_METHOD_NAME.to_string(), id);
        drop(notification_state);

        tokio::spawn(async move {
            while let Some(msg) = channel_recv.recv().await {
                let msg = msg.into_data();

                match serde_json::from_slice(&msg) {
                    Ok(val) => {
                        let result: jsonrpc::JsonResponse = val;

                        println!("got notify_block resust\n notify block > \t\t{:?}", result);
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

        let request = jsonrpc::JsonRequest {
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

// impl futures::Future for Client {
//     type Output = mpsc::Receiver<u64>;

//     fn poll(self: futures::prelude::future::p Pin<&mut Self>, cx: &mut Context<'_>)
// }

impl Extension for Client {}
