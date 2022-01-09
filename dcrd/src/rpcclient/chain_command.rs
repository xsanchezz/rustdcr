use crate::dcrjson::cmd_types;

use {
    super::{
        check_config, client::Client, connection::RPCConn, error::RpcClientError, future_type,
    },
    crate::dcrjson::commands,
};

/// Generates clients command
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

    command_generator!(
        "get_block_verbose returns a data structure from the server with information
        about a block given its hash.",
        get_block_verbose,
        future_type::GetBlockVerboseFuture,
        commands::METHOD_GET_BLOCK,
        &[
            serde_json::json!(block_hash),
            serde_json::json!(true),
            serde_json::json!(verbose_tx)
        ],
        block_hash: String,
        verbose_tx: bool
    );

    command_generator!(
        "decode_raw_transaction returns information about a transaction given its serialized bytes.",
        decode_raw_transaction,
        future_type::DecodeRawTransactionFuture,
        commands::METHOD_DECODE_RAW_TRANSACTION,
        &[serde_json::json!(serialized_tx)],
        serialized_tx: &[u8]
     );

    command_generator!(
        "estimate_smart_fee returns an estimation of a transaction fee rate (in dcr/KB)
         that new transactions should pay if they desire to be mined in up to
         'confirmations' blocks and the block number where the estimate was found.
        
         The mode parameter (roughly) selects the different thresholds for accepting
         an estimation as reasonable, allowing users to select different trade-offs
         between probability of the transaction being mined in the given target
         confirmation range and minimization of fees paid.
        
         As of 2019-01, only the default conservative mode is supported by dcrd.",
        estimate_smart_fee,
        future_type::EstimateSmartFeeFuture,
        commands::METHOD_ESTIMATE_SMART_FEE,
        &[serde_json::json!(confirmations), serde_json::json!(mode),],
        confirmations: i64,
        mode: cmd_types::EstimateSmartFeeMode
    );
}
