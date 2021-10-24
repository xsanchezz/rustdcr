//! JSON Errors.
//! Contains all possible JSON error for RPC connection.

/// RPC Json errors.
pub enum RpcServerError {
    /// Error marshalling server response.
    Marshaller(serde_json::Error),
    /// Empty response returned by server.
    EmptyResponse,
    /// Invalid response
    InvalidResponse(String),
    /// Error returned to client by server.
    ServerError(super::chain_command_result::RpcError),
}

impl std::fmt::Display for RpcServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RpcServerError::EmptyResponse => write!(f, "Empty response from server."),
            RpcServerError::InvalidResponse(ref e) => {
                write!(f, "Invalid response from server, error: {}.", e)
            }
            RpcServerError::Marshaller(ref e) => write!(f, "Marshaller error: {}.", e),
            RpcServerError::ServerError(ref e) => write!(f, "Server returned an error: {:?}.", e),
        }
    }
}

impl std::fmt::Debug for RpcServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RpcServerError::EmptyResponse => {
                write!(f, "RpcServerError(Empty response from server)")
            }
            RpcServerError::InvalidResponse(ref e) => write!(
                f,
                "RpcServerError(Invalid response from server, error: {})",
                e
            ),
            RpcServerError::Marshaller(ref e) => {
                write!(f, "RpcServerError(Marshaller error: {})", e)
            }
            RpcServerError::ServerError(ref e) => {
                write!(f, "RpcServerError(Server returned an error: {:?})", e)
            }
        }
    }
}
