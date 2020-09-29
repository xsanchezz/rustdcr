//! RPC Types.
//! Decred JSON RPC notification commands. Also contains standard commands to interact with lower versions such
//! as bitcoind.

/// Notification from the chain server that a block has been connected.
pub(super) const BLOCK_CONNECTED_NOTIFICATION_METHOD: &str = "blockconnected";
/// Notification from the chain server that a block has been disconnected.
pub(super) const BLOCK_DISCONNECTED_NOTIFICATION_METHOD: &str = "blockdisconnected";
/// Issues a notify blocks command to RPC server.
pub(super) const NOTIFY_BLOCKS_METHOD: &str = "notifyblocks";
/// Issues a notify on new tickets command to RPC server.
pub(super) const NOTIFY_NEW_TICKETS_METHOD: &str = "notifynewtickets";
/// Notifies a client when new tickets have matured.
pub(super) const NEW_TICKETS_NOTIFICATION_METHOD: &str = "newtickets";

/// Implements JSON RPC request structure to server.
#[derive(serde::Serialize)]
pub(super) struct JsonRequest<'a> {
    pub(super) jsonrpc: &'a str,
    pub(super) id: u64,
    pub(super) method: &'a str,
    pub(super) params: &'a [serde_json::Value],
}

/// Implements JSON RPC response structure from server.
#[derive(serde::Deserialize, Default, Debug)]
#[serde(default)]
pub(crate) struct JsonResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: serde_json::Value,
    pub result: serde_json::Value,
    pub params: serde_json::Value,
    pub error: serde_json::Value,
}
