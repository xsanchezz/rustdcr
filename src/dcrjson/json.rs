// {"jsonrpc":"1.0","result":103789,"error":null,"id":2}

// pub struct JSONMessageIn{
//     pub jsonrpc: serde_json::Number,
//     result: serde_json::
// }

#[derive(serde::Serialize, serde::Deserialize)]
pub struct JsonID {
    pub id: u64,
}
