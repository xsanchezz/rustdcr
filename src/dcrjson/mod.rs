mod chain_commands;
mod errors;
pub use errors::RpcJsonError;
pub mod future_types;
pub mod rpc_types;
pub use chain_commands::ChainCommand;

use log::warn;

/// Parse hex string to bytes
pub(self) fn parse_hex_parameters(value: &serde_json::Value) -> Option<Vec<u8>> {
    if value.is_null() {
        return Some(Vec::new());
    }

    let s: String = match serde_json::from_value(value.clone()) {
        Ok(val) => val,

        Err(e) => {
            warn!("Error unmarshalling hex parameters, error: {}", e);
            return None;
        }
    };

    match ring::test::from_hex(s.as_str()) {
        Ok(v) => return Some(v),

        Err(e) => {
            warn!("Error converting unmarshalled string to hex, error: {}", e);
            return None;
        }
    };
}
