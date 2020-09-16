use std::collections::HashMap;

const HashSize: i8 = 32;

pub struct NotificationHandlers {
    OnClientConnected: Option<fn()>,

    OnBlockConnected: Option<fn(block_header: [u8], transactions: Vec<[u8]>)>,

    OnBlockDisconnected: Option<fn(block_header: [u8])>,

    OnWork: Option<fn(data: [u8], target: [u8], reason: String)>,

    OnRelevantTxAccepted: Option<fn(transaction: [u8])>,

    OnReorganization: Option<
        fn(old_hash: &[u8; HashSize], old_height: i32, new_hash: &[u8; HashSize], new_height: i32),
    >,

    OnWinningTickets: Option<fn(block_hash: i64, tickets: Vec<&[u8; HashSize]>)>,

    OnSpentAndMissedTickets: Option<
        fn(
            hash: &[u8; HashSize],
            height: i64,
            stake_diff: i64,
            tickets: HashMap<[u8; HashSize], bool>,
        ),
    >,

    OnNewTickets: Option<fn(height: i64, stake_diff: i64, tickets: Vec<&[u8; HashSize]>)>,

    OnStakeDifficulty: Option<fn(hash: &[u8; HashSize], height: i64, stake_diff: i64)>,
    //  OnTxAccepted: Option<fn(hash: &[u8;HashSize], amount dcrutil.Amount)>
    OnUnknownNotification: Option<fn(method: String, params: [u8])>,
}
