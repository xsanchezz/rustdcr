//! RPC Types.
//! Decred JSON RPC notification commands. Also contains standard commands to interact with lower versions such
//! as bitcoind.

/// Notification from the chain server that a block has been connected.
pub(crate) const BLOCK_CONNECTED_NOTIFICATION_METHOD: &str = "blockconnected";
/// Notification from the chain server that a block has been disconnected.
pub(crate) const BLOCK_DISCONNECTED_NOTIFICATION_METHOD: &str = "blockdisconnected";
/// Notifies a client when new tickets have matured.
pub(crate) const NEW_TICKETS_NOTIFICATION_METHOD: &str = "newtickets";
/// Notification that a new block has been generated.
pub(crate) const WORK_NOTIFICATION_METHOD: &str = "work";

/// Issues a notify blocks command to RPC server.
pub(crate) const NOTIFY_BLOCKS_METHOD: &str = "notifyblocks";
/// Issues a notify on new tickets command to RPC server.
pub(crate) const NOTIFY_NEW_TICKETS_METHOD: &str = "notifynewtickets";
/// Registers the client to receive notifications when a new block template has been generated
pub(crate) const NOTIFIY_NEW_WORK_METHOD: &str = "notifywork";

/// Implements JSON RPC request structure to server.
#[derive(serde::Serialize)]
pub(crate) struct JsonRequest<'a> {
    pub jsonrpc: &'a str,
    pub id: u64,
    pub method: &'a str,
    pub params: &'a [serde_json::Value],
}

/// Implements JSON RPC response structure from server.
#[derive(serde::Deserialize, Default, Debug)]
#[serde(default)]
pub(crate) struct JsonResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: serde_json::Value,
    pub result: serde_json::Value,
    pub params: Vec<serde_json::Value>,
    pub error: serde_json::Value,
}
