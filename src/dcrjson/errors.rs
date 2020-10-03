//! JSON Errors.
//! Contains all possible JSON error for RPC connection.

/// RPC Json errors.
pub enum RpcServerError {
    /// Error marshalling server response.
    Marshaller(serde_json::Error),
    /// Empty response returned by server.
    EmptyResponse,
    /// Error returned to client by server.
    ServerError(String),
}

impl std::fmt::Display for RpcServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RpcServerError::EmptyResponse => write!(f, "Empty response by server."),
            RpcServerError::Marshaller(ref e) => write!(f, "Marshaller error: {}.", e),
            RpcServerError::ServerError(ref e) => write!(f, "Server returned an error: {}.", e),
        }
    }
}

impl std::fmt::Debug for RpcServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RpcServerError::EmptyResponse => write!(f, "RpcServerError(Empty response by server)"),
            RpcServerError::Marshaller(ref e) => {
                write!(f, "RpcServerError(Marshaller error: {})", e)
            }
            RpcServerError::ServerError(ref e) => {
                write!(f, "RpcServerError(Server returned an error: {})", e)
            }
        }
    }
}
