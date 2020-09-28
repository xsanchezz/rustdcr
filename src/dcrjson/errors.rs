pub enum Error {
    /// An rpcclient error.
    WebsocketDisabled,
    /// Unregisted on server notification callback.
    UnregisteredNotification(String),
    /// Error marshalling server response.
    Marshaller(serde_json::Error),
    /// On websocket channel closure.
    WebsocketClose,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::WebsocketDisabled => write!(f, ""),
            Error::UnregisteredNotification(ref e) => write!(f, ""),
            Error::Marshaller(ref e) => write!(f, ""),
            Error::WebsocketClose => write!(f, ""),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::WebsocketDisabled => write!(f, ""),
            Error::UnregisteredNotification(ref e) => write!(f, ""),
            Error::Marshaller(ref e) => write!(f, ""),
            Error::WebsocketClose => write!(f, ""),
        }
    }
}
