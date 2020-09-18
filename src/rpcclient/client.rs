use crate::{
    helper::{error_helper, waitgroup},
    rpcclient::{connection, constants, notify},
};
use log::{info, trace, warn};
use std::{
    error,
    sync::{mpsc, Arc, Condvar, Mutex, RwLock},
};
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
    let client = match config.disable_connect_on_new {
        false => {
            info!("Dialing RPC using websocket to host {}", config.host);

            let websokcet = match config.dial_websocket() {
                Ok(websocket) => websocket,

                Err(e) => return Err(e),
            };

            let (conn_established_send, conn_established_recv) = mpsc::channel();
            conn_established_send.send(true);

            Client {
                websocket_connection: Some(Mutex::new(websokcet)),

                disconnected: RwLock::new(config.disable_connect_on_new),

                configuration: RwLock::new(config),
                notification_handler: RwLock::new(notif_handler),

                connection_established: (conn_established_send, conn_established_recv),
                disconnect: mpsc::channel(),
                shutdown: mpsc::channel(),

                waitgroup: waitgroup::WaitGroup::new(),
            }
        }

        true => Client {
            websocket_connection: None,

            disconnected: RwLock::new(config.disable_connect_on_new),

            configuration: RwLock::new(config),
            notification_handler: RwLock::new(notif_handler),

            connection_established: mpsc::channel(),
            disconnect: mpsc::channel(),
            shutdown: mpsc::channel(),

            waitgroup: waitgroup::WaitGroup::new(),
        },
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
/// All field in `Client` are async safe.
pub struct Client {
    /// websocket connection to the underlying server, it is protected by a mutex lock.
    websocket_connection: Option<Mutex<WebSocket<AutoStream>>>, // ToDo: This could be expensive.

    /// Holds the connection configuration associated with the client.
    configuration: RwLock<connection::ConnConfig>,

    /// Contains all notification callback functions. It is protected by a mutex lock.
    notification_handler: RwLock<notify::NotificationHandlers>,

    /// disconnected indicates whether the client is disconnected from the server.
    disconnected: RwLock<bool>,

    /// connection_established is a network infrastructure that notifies all channel
    /// when the RPC serve is connected or disconnected.
    connection_established: (mpsc::Sender<bool>, mpsc::Receiver<bool>),

    /// disconnect is a channel to websocket to disconnect from server.
    disconnect: (mpsc::Sender<bool>, mpsc::Receiver<bool>),

    /// Broadcast shutdown command to channels so as to disconnect RPC server.
    shutdown: (mpsc::Sender<bool>, mpsc::Receiver<bool>),

    /// Asynchronously blocks.
    waitgroup: waitgroup::WaitGroup,
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
        self.waitgroup.add(1);

        trace!("Starting RPC client");

        match self.notification_handler.read() {
            Ok(guard) => guard.on_client_connected.and_then(|on_client_connected| {
                on_client_connected();
                None::<bool>
            }),

            Err(_) => None, // ToDo: Return Err.
        };
    }

    pub fn wait_for_shutdown(&self) {
        self.waitgroup.wait();
    }

    pub fn shutdown(&self) {
        let mut ws = match &self.websocket_connection {
            Some(ws_lock) => match ws_lock.lock() {
                Ok(ws) => ws,

                Err(e) => {
                    warn!("error closing websocket on shutdown, err: {}", e);
                    self.waitgroup.wait();
                    return;
                }
            },

            None => {
                self.waitgroup.wait();
                return;
            }
        };

        match ws.close(Some(tungstenite::protocol::CloseFrame {
            reason: "Normal Closure".into(),
            code: 1000.into(),
        })) {
            Ok(_) => loop {
                match ws.write_pending() {
                    Ok(_) => {
                        break;
                    }

                    Err(e) => warn!(
                        "error retrieve close success from websocket server, err: {}",
                        e
                    ),
                }
            },

            Err(e) => warn!("error sending close command to websocket, err: {}", e),
        };

        self.waitgroup.done();
    }
}
