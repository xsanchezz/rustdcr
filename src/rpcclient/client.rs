use crate::{
    helper::error_helper,
    rpcclient::{connection, constants, notify},
};
use log::{debug, info, trace, warn};
use std::{error, sync};
use tungstenite::{client::AutoStream, WebSocket};

/// Creates a new RPC client based on the provided connection configuration
/// details.  The notification handlers parameter may be None if you are not
/// interested in receiving notifications and will be ignored if the
/// configuration is set to run in HTTP POST mode.
///
pub fn new(
    mut config: connection::ConnConfig,
    notif_handler: notify::NotificationHandlers,
) -> Result<Client, String> {
    let websokcet_connection = match config.disable_connect_on_new {
        false => {
            println!("Dialing RPC using websocket to host {}", config.host);

            match config.dial_websocket() {
                Ok(websocket) => Some(websocket),

                Err(e) => return Err(e),
            }
        }

        true => None,
    };

    let (sender, receiver) = sync::mpsc::channel::<bool>();

    if !config.disable_connect_on_new {
        match sender.send(true) {
            Ok(_) => {
                println!("Established connection to RPC server {}", config.host);
            }

            Err(e) => return Err(error_helper::new(constants::ERR_STARTING_CHANNEL, e.into())),
        }
    }

    let client = Client {
        websocket_connection: websokcet_connection,

        disconnected: sync::RwLock::new(config.disable_connect_on_new),

        configuration: config,
        notification_handler: sync::Mutex::new(notif_handler),

        connection_established: (sender, receiver),
        disconnect: sync::mpsc::channel(),
        shutdown: sync::mpsc::channel(),
    };

    client.start();

    Ok(client)
}

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
    websocket_connection: Option<WebSocket<AutoStream>>,

    configuration: connection::ConnConfig,
    notification_handler: sync::Mutex<notify::NotificationHandlers>,

    disconnected: sync::RwLock<bool>,

    connection_established: (sync::mpsc::Sender<bool>, sync::mpsc::Receiver<bool>),
    disconnect: (sync::mpsc::Sender<bool>, sync::mpsc::Receiver<bool>),
    shutdown: (sync::mpsc::Sender<bool>, sync::mpsc::Receiver<bool>),
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

    /// start begins processing input and output messages.
    pub(crate) fn start(&self) {
        trace!("Starting RPC client");

        match self.notification_handler.lock() {
            Ok(guard) => guard.on_client_connected.and_then(|on_client_connected| {
                on_client_connected();
                None::<bool>
            }),

            Err(_) => None, // ToDo: Return Err.
        };
    }
}
