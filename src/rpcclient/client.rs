use crate::rpcclient::{connection, notify};
use std::{error, sync};

/// Creates a new RPC client based on the provided connection configuration
/// details.  The notification handlers parameter may be None if you are not
/// interested in receiving notifications and will be ignored if the
/// configuration is set to run in HTTP POST mode.
///
pub fn new(_: &connection::ConnConfig, _: Option<&notify::NotificationHandlers>) {}

/// Represents a Decred RPC client which allows easy access to the
/// various RPC methods available on a Decred RPC server.  Each of the wrapper
/// functions handle the details of converting the passed and return types to and
/// from the underlying JSON types which are required for the JSON-RPC
/// invocations
///
/// The client provides each RPC in both synchronous (blocking) and asynchronous
/// (non-blocking) forms.  The asynchronous forms are based on the concept of
/// futures where they return an instance of a type that promises to deliver the
/// result of the invocation at some future time.  Invoking the Receive method on
/// the returned future will block until the result is available if it's not
/// already.
///
pub struct Client {
    pub(crate) connection: sync::Mutex<connection::ConnConfig>,
    pub(crate) notification_handler: sync::Mutex<notify::NotificationHandlers>,

    pub(crate) connection_established: sync::RwLock<bool>,
    pub(crate) disconnected: sync::RwLock<bool>,
    pub(crate) shutdown: sync::RwLock<bool>,
}

impl Client {
    /// Establishes the initial websocket connection.  This is necessary when
    /// a client was created after setting the DisableConnectOnNew field of the
    // Config struct.
    ///
    /// If the connection fails and retry is true, this method will continue to try
    /// reconnections with backoff until the context is done.
    ///
    /// This method will error if the client is not configured for websockets, if the
    /// connection has already been established, or if none of the connection
    /// attempts were successful. The client will be shut down when the passed
    /// context is terminated.
    ///
    pub fn connect(&self) -> Result<(), Box<dyn error::Error>> {
        todo!()
    }
}
