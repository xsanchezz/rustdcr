//! JSON Errors.
//! Contains all possible JSON error for RPC connection.

/// RPC Json errors.
pub enum RpcJsonError {
    /// An rpcclient error.
    WebsocketDisabled,
    /// Unregisted on server notification callback.
    UnregisteredNotification(String),
    /// Error marshalling server response.
    Marshaller(serde_json::Error),
    /// On websocket channel closure.
    WebsocketClosed,
    /// Empty response returned by server.
    EmptyResponse,
}

impl std::fmt::Display for RpcJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RpcJsonError::WebsocketDisabled => {
                write!(f, "JSON command requires websocket connection.")
            }
            RpcJsonError::UnregisteredNotification(ref e) => {
                write!(f, "Unregistered notification callback, type: {}", e)
            }
            RpcJsonError::EmptyResponse => write!(f, "Empty response by server"),
            RpcJsonError::Marshaller(ref e) => write!(f, "Marshaller error: {}", e),
            RpcJsonError::WebsocketClosed => write!(f, "Websocket connection closed."),
        }
    }
}

impl std::fmt::Debug for RpcJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RpcJsonError::WebsocketDisabled => write!(
                f,
                "RpcJsonError(JSON command requires websocket connection)"
            ),
            RpcJsonError::UnregisteredNotification(ref e) => write!(
                f,
                "RpcJsonError(Unregistered notification callback, type: {})",
                e
            ),
            RpcJsonError::EmptyResponse => write!(f, "RpcJsonError(Empty response by server)"),
            RpcJsonError::Marshaller(ref e) => write!(f, "RpcJsonError(Marshaller error: {})", e),
            RpcJsonError::WebsocketClosed => write!(f, "RpcJsonError(Websocket connection closed)"),
        }
    }
}
