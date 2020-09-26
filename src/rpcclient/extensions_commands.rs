#[deny(missing_docs)]
use super::client::Client;
use tokio::sync::mpsc;

use core::future::Future;

pub trait Extension {
    fn add_node(&self) {}

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

struct AddNode {}

// impl futures::Future for Client {
//     type Output = mpsc::Receiver<u64>;

//     fn poll(self: futures::prelude::future::p Pin<&mut Self>, cx: &mut Context<'_>)
// }

impl Extension for Client {
    fn add_node(&self) {}

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
