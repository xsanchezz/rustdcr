//! RPC Types.
//! Decred JSON RPC notification commands. Also contains standard commands to interact with lower versions such
//! as bitcoind.

/// Notifies that a block has been connected.
pub(crate) const NOTIFICATION_METHOD_BLOCK_CONNECTED: &str = "blockconnected";
/// Notifies that a block has been disconnected.
pub(crate) const NOTIFICATION_METHOD_BLOCK_DISCONNECTED: &str = "blockdisconnected";
/// Notifies a client when new tickets have matured.
pub(crate) const NOTIFICATION_METHOD_NEW_TICKETS: &str = "newtickets";
/// Notifies that a new block has been generated.
pub(crate) const NOTIFICATION_METHOD_WORK: &str = "work";
/// Notifies when a new transaction has been accepted and the client has
/// requested standard transaction details.
pub(crate) const NOTIFICATION_METHOD_TX_ACCEPTED: &str = "txaccepted";
/// Notifies when a new transaction has been accepted and the client
/// has requested verbose transaction details.
pub(crate) const NOTIFICATION_METHOD_TX_ACCEPTED_VERBOSE: &str = "txacceptedverbose";
/// Notifies a client when the stake difficulty has been updated
pub(crate) const NOTIFICATION_METHOD_STAKE_DIFFICULTY: &str = "stakedifficulty";
/// Notifies that the block chain is in the process of a reorganization.
pub(crate) const NOTIFICATION_METHOD_REORGANIZATION: &str = "reorganization";

/// Issues a notify blocks command to RPC server.
pub(crate) const METHOD_NOTIFY_BLOCKS: &str = "notifyblocks";
/// Issues a notify on new tickets command to RPC server.
pub(crate) const METHOD_NOTIFY_NEW_TICKETS: &str = "notifynewtickets";
/// Registers the client to receive notifications when a new block template has been generated
pub(crate) const METHOD_NOTIFIY_NEW_WORK: &str = "notifywork";
/// Registers the client to receive either a txaccepted or a txacceptedverbose notification
/// when a new transaction is accepted into the mempool.
pub(crate) const METHOD_NEW_TX: &str = "notifynewtransactions";
/// Registers the client to receive a stakedifficulty notification when the stake difficulty is updated.
pub(crate) const METHOD_STAKE_DIFFICULTY: &str = "notifystakedifficulty";

/// Returns information about the current state of the block chain.
pub(crate) const METHOD_GET_BLOCKCHAIN_INFO: &str = "getblockchaininfo";
/// Returns the number of blocks in the longest block chain.
pub(crate) const METHOD_GET_BLOCK_COUNT: &str = "getblockcount";
/// Returns hash of the block in best block chain at the given height.
pub(crate) const METHOD_GET_BLOCK_HASH: &str = "getblockhash";
