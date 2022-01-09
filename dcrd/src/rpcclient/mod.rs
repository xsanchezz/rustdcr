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

macro_rules! check_config {
    ($self:ident) => {
        if $self.conn.is_http_mode() || $self.is_disconnected().await {
            return Err(RpcClientError::RpcDisconnected);
        }
    };
}

pub(super) use check_config;
