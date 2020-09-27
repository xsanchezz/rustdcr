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
    pub result: serde_json::Value,
    pub params: serde_json::Value,
    pub error: serde_json::Value,
}
