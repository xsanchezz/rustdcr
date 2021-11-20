use {
    super::{client::Client, create_command, error::RpcClientError, future_type},
    crate::dcrjson::commands,
};

impl Client {
    create_command!(
        get_blockchain_info,
        future_type::GetBlockchainInfoFuture,
        commands::METHOD_GET_BLOCKCHAIN_INFO,
        &[],
    );

    create_command!(
        get_block_count,
        future_type::GetBlockCountFuture,
        commands::METHOD_GET_BLOCK_COUNT,
        &[],
    );

    create_command!(
        get_block_hash,
        future_type::GetBlockHashFuture,
        commands::METHOD_GET_BLOCK_HASH,
        &[serde_json::json!(block_height)],
        block_height: i64
    );
}
