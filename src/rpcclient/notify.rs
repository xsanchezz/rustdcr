#[deny(missing_docs)]
use {crate::rpcclient::constants, std::collections::HashMap};

/// NotificationHandlers defines callback function pointers to invoke with
/// notifications.  Since all of the functions are None by default, all
/// notifications are effectively ignored until their handlers are set to a
/// concrete callback.
///
/// All callback functions are run async and are safe from blocking client requests.
#[derive(Default)]
pub struct NotificationHandlers {
    /// on_client_connected callback function is invoked when the client connects or
    /// reconnects to the RPC server.
    pub on_client_connected: Option<fn()>,

    /// on_block_connected callback function is invoked when a block is connected to the
    /// longest `best` chain.
    pub on_block_connected: Option<fn(block_header: Vec<u8>, transactions: Vec<Vec<u8>>)>,

    /// on_block_disconnected callback function is invoked when a block is disconnected from
    /// the longest `best` chain.
    pub on_block_disconnected: Option<fn(block_header: [u8])>,

    /// on_work callback function is invoked when a new block template is generated.
    pub on_work: Option<fn(data: [u8], target: [u8], reason: String)>,

    /// on_relevant_tx_accepted callback function is invoked when an unmined transaction passes
    /// the client's transaction filter.
    pub on_relevant_tx_accepted: Option<fn(transaction: [u8])>,

    /// on_reorganization callback function is invoked when the blockchain begins reorganizing.
    pub on_reorganization: Option<
        fn(
            old_hash: &[u8; constants::HASH_SIZE],
            old_height: i32,
            new_hash: &[u8; constants::HASH_SIZE],
            new_height: i32,
        ),
    >,

    /// on_winning_tickets callback function is invoked when a block is connected and eligible tickets
    /// to be voted on for this chain are given.
    pub on_winning_tickets:
        Option<fn(block_hash: i64, tickets: Vec<&[u8; crate::rpcclient::constants::HASH_SIZE]>)>,

    /// on_spent_and_missed_tickets callback function is invoked when a block is connected to the
    /// longest `best` chain and tickets are spent or missed.
    pub on_spent_and_missed_tickets: Option<
        fn(
            hash: &[u8; constants::HASH_SIZE],
            height: i64,
            stake_diff: i64,
            tickets: HashMap<[u8; constants::HASH_SIZE], bool>,
        ),
    >,

    /// on_new_tickets callback function is invoked when a block is connected to the longest `best` chain
    /// and tickets have matured and become active.
    pub on_new_tickets:
        Option<fn(height: i64, stake_diff: i64, tickets: Vec<&[u8; constants::HASH_SIZE]>)>,

    /// on_stake_difficulty callback function is invoked when a block is connected to the longest `best` chain
    /// and a new difficulty is calculated.
    pub on_stake_difficulty:
        Option<fn(hash: &[u8; constants::HASH_SIZE], height: i64, stake_diff: i64)>,

    /// on_unknown_notification callback function is invoked when an unrecognized notification is received.
    /// This typically means the notification handling code for this package needs to be updated for a new
    /// notification type or the caller is using a custom notification this package does not know about.
    pub on_unknown_notification: Option<fn(method: String, params: [u8])>,
}

/// Used to track the current state of successfully registered notifications so the state can be automatically
// re-established on reconnect.
/// On notification registration, message sent to the RPC server is copied and stored. This is so that on reconnection
/// same message can be sent to the server and server can reply to recently registered command channel which calls the callback
/// function.
#[derive(Default)]
pub(crate) struct NotificationState {
    pub(crate) notify_blocks: Option<u64>,
    pub(crate) notify_work: Option<u64>,
    pub(super) notify_winning_tickets: Option<u64>,
    pub(crate) notify_spent_and_missed_tickets: Option<u64>,
    pub(crate) notify_new_tickets: Option<u64>,
    pub(crate) notify_stake_difficulty: Option<u64>,
    pub(crate) notify_new_tx: Option<u64>,
    pub(crate) notify_new_tx_verbose: Option<u64>,
}
