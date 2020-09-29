use dcrdrs::{
    dcrutil::app_data,
    rpcclient::{client, connection, notify},
};
use std::{fs, path::PathBuf};

use slog;
use slog_scope;
use slog_stdlog;
use slog_term;

use std::fs::OpenOptions;

use slog::Drain;

#[tokio::main]
async fn main() {
    let log_path = "rpcclient.log";
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();

    // create logger
    let decorator = slog_term::PlainSyncDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let logger = slog::Logger::root(drain, slog::o!());

    // slog_stdlog uses the logger from slog_scope, so set a logger there
    let _guard = slog_scope::set_global_logger(logger);

    // register slog_stdlog as the log handler with the log crate
    slog_stdlog::init().unwrap();

    // Get dcrd app directory, if none is found use current path.
    let mut app_dir = match app_data::get_app_data_dir("dcrd".into(), false) {
        Some(dir) => dir,

        None => PathBuf::new().join("."),
    };

    app_dir.push("rpc.cert");

    let certs = fs::read_to_string(app_dir).unwrap();

    let config = connection::ConnConfig {
        host: "127.0.0.1:9109".to_string(),
        certificates: certs,
        password: "rpcpassword".to_string(),
        user: "rpcuser".to_string(),
        ..Default::default()
    };

    let notif_handler = notify::NotificationHandlers {
        on_client_connected: Some(|| {
            println!("client connected");
        }),

        on_block_connected: Some(|block_header: Vec<u8>, transactions: Vec<Vec<u8>>| {
            println!(
                "\n\n\n\t\t\t\tBlock Connected Notif\nBlock header: {:?} \n\nTransactions: {:?}\n\n\n",
                block_header, transactions,
            )
        }),

        on_block_disconnected: Some(|block_header: Vec<u8>| {
            println!(
                "\n\n\t\t\t\tBlock Disconnected Notif\n\nBlock header: {:?}\n\n\n",
                block_header,
            )
        }),

        ..Default::default()
    };

    let mut client = client::new(config, notif_handler).await.unwrap();

    client.notify_blocks().await.unwrap();

    client.wait_for_shutdown();
}
