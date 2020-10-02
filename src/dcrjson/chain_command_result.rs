use std::collections::HashMap;

#[derive(serde::Deserialize, Default)]
pub struct AgendaInfo {
    pub status: String,
    #[serde(default)]
    pub since: i64,
    #[serde(rename = "starttime")]
    pub start_time: u64,
    #[serde(rename = "expiretime")]
    pub expire_time: u64,
}

#[derive(serde::Deserialize, Default)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: i64,
    pub headers: i64,
    #[serde(rename = "syncheight")]
    pub sync_height: i64,
    #[serde(rename = "bestblockhash")]
    pub best_block_hash: String,
    pub difficulty: u32,
    #[serde(rename = "difficultyratio")]
    pub difficulty_ratio: f64,
    #[serde(rename = "verificationprogress")]
    pub verification_progress: f64,
    #[serde(rename = "chainwork")]
    pub chain_work: String,
    #[serde(rename = "initialblockdownload")]
    pub initial_block_download: bool,
    #[serde(rename = "maxblocksize")]
    pub max_block_size: i64,
    pub deployments: HashMap<String, AgendaInfo>,
}

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
