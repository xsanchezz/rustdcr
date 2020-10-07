use {
    super::{client::Client, RpcClientError},
    crate::dcrjson::{future_types, rpc_types},
};

impl Client {
    pub async fn get_blockchain_info(
        &mut self,
    ) -> Result<future_types::GetBlockchainInfoFuture, RpcClientError> {
        let config = self.configuration.read().await;

        // Error if user is not on HTTP mode and websocket is disconnected.
        if !config.http_post_mode && self.is_disconnected().await {
            return Err(RpcClientError::RpcDisconnected);
        }
        drop(config);

        let cmd_result = self
            .send_custom_command(rpc_types::METHOD_GET_BLOCKCHAIN_INFO, &[])
            .await;

        match cmd_result {
            Ok(e) => return Ok(future_types::GetBlockchainInfoFuture { message: e.1 }),

            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn get_block_count(
        &mut self,
    ) -> Result<future_types::GetBlockCountFuture, RpcClientError> {
        let config = self.configuration.read().await;

        // Error if user is not on HTTP mode and websocket is disconnected.
        if !config.http_post_mode && self.is_disconnected().await {
            return Err(RpcClientError::RpcDisconnected);
        }
        drop(config);

        let cmd_result = self
            .send_custom_command(rpc_types::METHOD_GET_BLOCK_COUNT, &[])
            .await;

        match cmd_result {
            Ok(e) => return Ok(future_types::GetBlockCountFuture { message: e.1 }),

            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn get_block_hash(
        &mut self,
        block_height: i64,
    ) -> Result<future_types::GetBlockHashFuture, RpcClientError> {
        let config = self.configuration.read().await;

        // Error if user is not on HTTP mode and websocket is disconnected.
        if !config.http_post_mode && self.is_disconnected().await {
            return Err(RpcClientError::RpcDisconnected);
        }
        drop(config);

        let cmd_result = self
            .send_custom_command(
                rpc_types::METHOD_GET_BLOCK_HASH,
                &[serde_json::json!(block_height)],
            )
            .await;

        match cmd_result {
            Ok(e) => return Ok(future_types::GetBlockHashFuture { message: e.1 }),

            Err(e) => {
                return Err(e);
            }
        }
    }
}
