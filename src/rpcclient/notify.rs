use crate::rpcclient::constants;
use std::collections::HashMap;

/// NotificationHandlers defines callback function pointers to invoke with
/// notifications.  Since all of the functions are None by default, all
/// notifications are effectively ignored until their handlers are set to a
/// concrete callback.
pub struct NotificationHandlers {
    pub on_client_connected: Option<fn()>,

    pub on_block_connected: Option<fn(block_header: Vec<u8>, transactions: Vec<Vec<u8>>)>,

    pub on_block_disconnected: Option<fn(block_header: [u8])>,

    pub on_work: Option<fn(data: [u8], target: [u8], reason: String)>,

    pub on_relevant_tx_accepted: Option<fn(transaction: [u8])>,

    pub on_reorganization: Option<
        fn(
            old_hash: &[u8; constants::HASH_SIZE],
            old_height: i32,
            new_hash: &[u8; constants::HASH_SIZE],
            new_height: i32,
        ),
    >,

    pub on_winning_tickets:
        Option<fn(block_hash: i64, tickets: Vec<&[u8; crate::rpcclient::constants::HASH_SIZE]>)>,

    pub on_spent_and_missed_tickets: Option<
        fn(
            hash: &[u8; constants::HASH_SIZE],
            height: i64,
            stake_diff: i64,
            tickets: HashMap<[u8; constants::HASH_SIZE], bool>,
        ),
    >,

    pub on_new_tickets:
        Option<fn(height: i64, stake_diff: i64, tickets: Vec<&[u8; constants::HASH_SIZE]>)>,

    pub on_stake_difficulty:
        Option<fn(hash: &[u8; constants::HASH_SIZE], height: i64, stake_diff: i64)>,

    //  OnTxAccepted: Option<fn(hash: &[u8;HashSize], amount dcrutil.Amount)>
    pub on_unknown_notification: Option<fn(method: String, params: [u8])>,
}

impl Default for NotificationHandlers {
    fn default() -> Self {
        NotificationHandlers {
            on_block_connected: None,
            on_block_disconnected: None,
            on_client_connected: None,
            on_new_tickets: None,
            on_relevant_tx_accepted: None,
            on_reorganization: None,
            on_spent_and_missed_tickets: None,
            on_stake_difficulty: None,
            on_unknown_notification: None,
            on_winning_tickets: None,
            on_work: None,
        }
    }
}
