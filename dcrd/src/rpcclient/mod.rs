#![cfg(feature = "rpcclient")]
pub mod chain_command;
pub mod chain_notification;
pub mod client;
pub mod connection;
pub(crate) mod constants;
pub mod error;
mod future_type;
mod infrastructure;
pub mod notify;
pub mod test;

macro_rules! create_command {
    ($name: ident, $output_type: ty, $command: expr, $json_params: expr, $($fn_params:ident : $fn_type: ty),*) => {
        pub async fn $name(&mut self, $($fn_params : $fn_type),*) -> Result<$output_type, RpcClientError> {
            // Error if user is not on HTTP mode and websocket is disconnected.
            {
                let config = self.configuration.read().await;

                if !config.http_post_mode && self.is_disconnected().await {
                    return Err(RpcClientError::RpcDisconnected);
                }
            }

            let cmd_result = self.send_custom_command($command, $json_params).await;

            match cmd_result {
                Ok(e) => Ok(<$output_type>::new(e.1)),

                Err(e) => Err(e),
            }
        }
    };
}

pub(super) use create_command;
