use super::connection::RPCConn;

use {
    super::{check_config, client::Client, error::RpcClientError, future_type},
    crate::dcrjson::commands,
};

macro_rules! command_generator {
    ($doc: tt, $name: ident, $output_type: ty, $command: expr, $json_params: expr, $($fn_params:ident : $fn_type: ty),*) => {
        #[doc = $doc]
        pub async fn $name(&mut self, $($fn_params : $fn_type),*) -> Result<$output_type, RpcClientError> {
            // Error if user is not on HTTP mode and websocket is disconnected.
            check_config!(self);

            let cmd_result = self.send_custom_command($command, $json_params).await;

            match cmd_result {
                Ok(e) => Ok(<$output_type>::new(e.1)),

                Err(e) => Err(e),
            }
        }
    };
}

impl<C: 'static + RPCConn> Client<C> {
    command_generator!(
        "get_blockchain_info returns information about the current state of the block chain.",
        get_blockchain_info,
        future_type::GetBlockchainInfoFuture,
        commands::METHOD_GET_BLOCKCHAIN_INFO,
        &[],
    );

    command_generator!(
        "get_block_count returns the number of blocks in the longest block chain.",
        get_block_count,
        future_type::GetBlockCountFuture,
        commands::METHOD_GET_BLOCK_COUNT,
        &[],
    );

    command_generator!(
        "get_block_hash returns the hash of the block in the best block chain at the given height.",
        get_block_hash,
        future_type::GetBlockHashFuture,
        commands::METHOD_GET_BLOCK_HASH,
        &[serde_json::json!(block_height)],
        block_height: i64
    );
}
