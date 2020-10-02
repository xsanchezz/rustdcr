use {
    super::client::Client,
    crate::dcrjson::{future_types, rpc_types, RpcJsonError},
};

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

    fn get_blockchain_info(&self) -> future_types::GetBlockchainInfoFuture;

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
impl Client {
    pub async fn get_blockchain_info(
        &mut self,
    ) -> Result<future_types::GetBlockchainInfoFuture, RpcJsonError> {
        let cmd = self
            .custom_command(rpc_types::METHOD_GET_BLOCKCHAIN_INFO, &[])
            .await;

        match cmd {
            Ok(e) => return Ok(future_types::GetBlockchainInfoFuture { message: e }),

            Err(e) => {
                return Err(e);
            }
        }
    }
}
