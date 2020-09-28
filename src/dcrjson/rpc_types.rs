pub(super) const BLOCK_CONNECTED_METHOD_NAME: &str = "blockconnected";
pub(super) const BLOCK_DISCONNECTED_METHOD_NAME: &str = "blockdisconnected";
pub(super) const NOTIFY_BLOCKS_METHOD_NAME: &str = "notifyblocks";

#[derive(serde::Serialize)]
pub(super) struct JsonRequest<'a> {
    pub(super) jsonrpc: &'a str,
    pub(super) id: u64,
    pub(super) method: &'a str,
    pub(super) params: &'a [serde_json::Value],
}

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
