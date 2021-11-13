//! Notification Handlers
//! On notification callback functions for websocket.

use {crate::chaincfg::chainhash::Hash, std::collections::HashMap};

/// NotificationHandlers defines callback function pointers to invoke with notifications.
/// Since all of the functions are None by default, all notifications are effectively
/// ignored until their handlers are set to a concrete callback.
///
/// NOTE: Unless otherwise documented, these handlers must NOT directly call any blocking calls
/// on the client instance since the input reader goroutine blocks until the callback has completed.
/// Doing so will result in a deadlock situation.
#[derive(Default)]
pub struct NotificationHandlers {
    /// on_client_connected callback function is invoked when the client connects or
    /// reconnects to the RPC server.
    pub on_client_connected: Option<fn()>,

    /// on_block_connected callback function is invoked when a block is connected to the
    /// longest `best` chain. It will only be invoked if a preceding call to
    /// NotifyBlocks has been made to register for the notification and the
    /// function is non-nil.
    pub on_block_connected: Option<fn(block_header: Vec<u8>, transactions: Vec<Vec<u8>>)>,

    /// on_block_disconnected callback function is invoked when a block is disconnected from
    /// the longest `best` chain.
    pub on_block_disconnected: Option<fn(block_header: Vec<u8>)>,

    /// on_work callback function is invoked when a new block template is generated.
    /// It will only be invoked if a preceding call to NotifyWork has
    /// been made to register for the notification and the function is non-nil.
    pub on_work: Option<fn(data: Vec<u8>, target: Vec<u8>, reason: String)>,

    /// on_relevant_tx_accepted callback function is invoked when an unmined transaction passes
    /// the client's transaction filter.
    pub on_relevant_tx_accepted: Option<fn(transaction: Vec<u8>)>,

    /// on_reorganization callback function is invoked when the blockchain begins reorganizing.
    /// It will only be invoked if a preceding call to NotifyBlocks has been made to register
    /// for the notification and the function is non-nil.
    pub on_reorganization:
        Option<fn(old_hash: Hash, old_height: i32, new_hash: Hash, new_height: i32)>,

    /// on_winning_tickets callback function is invoked when a block is connected and eligible tickets
    /// to be voted on for this chain are given. It will only be invoked if a
    /// preceding call to NotifyWinningTickets has been made to register for the
    /// notification and the function is non-nil.
    pub on_winning_tickets: Option<fn(block_hash: Hash, block_height: i64, tickets: Vec<Hash>)>,

    /// on_spent_and_missed_tickets callback function is invoked when a block is connected to the
    /// longest `best` chain and tickets are spent or missed. It will only be
    /// invoked if a preceding call to NotifySpentAndMissedTickets has been made to
    /// register for the notification and the function is non-nil.
    pub on_spent_and_missed_tickets:
        Option<fn(hash: Hash, height: i64, stake_diff: i64, tickets: HashMap<Hash, bool>)>,

    /// on_new_tickets callback function is invoked when a block is connected to the longest `best` chain
    /// and tickets have matured and become active. It will only be invoked
    /// if a preceding call to NotifyNewTickets has been made to register for the
    /// notification and the function is non-nil.
    pub on_new_tickets: Option<fn(hash: Hash, height: i64, stake_diff: i64, tickets: Vec<Hash>)>,

    /// on_tx_accepted is invoked when a transaction is accepted into the
    /// memory pool.  It will only be invoked if a preceding call to
    /// NotifyNewTransactions with the verbose flag set to false has been
    /// made to register for the notification and the function is non-nil.
    pub on_tx_accepted: Option<fn(hash: Hash, amount: crate::dcrutil::amount::Amount)>,

    /// Invoked when a transaction is accepted into the memory pool.
    /// It will only be invoked if a preceding call to notify_new_transactions
    /// with the verbose flag set to true has been made to register for
    /// the notification and the function is non-nil.
    pub on_tx_accepted_verbose: Option<fn(tx_details: crate::dcrjson::types::TxRawResult)>,

    /// on_stake_difficulty callback function is invoked when a block is connected
    /// to the longest `best` chain  and a new difficulty is calculated. It will only
    /// be invoked if a preceding call to NotifyStakeDifficulty has been
    /// made to register for the notification and the function is non-nil.
    pub on_stake_difficulty: Option<fn(hash: Hash, height: i64, stake_diff: i64)>,

    /// on_unknown_notification callback function is invoked when an unrecognized notification is received.
    /// This typically means the notification handling code for this package needs to be updated for a new
    /// notification type or the caller is using a custom notification this package does not know about.
    pub on_unknown_notification:
        Option<fn(method: String, params: crate::dcrjson::types::JsonResponse)>,
}
