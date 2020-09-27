use dcrdrs::{
    dcrutil::app_data,
    rpcclient::{client, connection, notify},
};
use std::{fs, future::Future, path::PathBuf};

#[macro_use]
use slog;
use slog_scope;
use slog_stdlog;
use slog_term;

use std::fs::OpenOptions;

use slog::Drain;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    let log_path = "your_log_file_path.log";
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
        certificates: certs,
        password: "admin".to_string(),
        user: "admin".to_string(),
        ..Default::default()
    };

    let notif_handler = notify::NotificationHandlers {
        on_client_connected: Some(|| {
            println!("client connected");
        }),

        on_block_connected: Some(|block_header: Vec<u8>, transactions: Vec<Vec<u8>>| {
            println!(
                "block header: {:?} \n\n transactions: {:?}",
                block_header, transactions,
            )
        }),

        ..Default::default()
    };

    let mut client = client::new(config, notif_handler).await.unwrap();
    // let ll = Arc::new(client);
    // let clone_cli = Arc::clone(ll.clone());
    //  task::Poll::is_ready(shutdown(&client));
    println!("waiting for shutdown!!!");

    client.notify_blocks().await.unwrap();

    // tokio::future::poll_fn(a);

    // let b = a.await;

    // let handle = &client.is_disconnected();

    // tokio::spawn(async move {
    //     tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;
    //     println!("is websocket disconnected, {}", handle.await);
    // });

    // client.shutdown().await;

    tokio::time::delay_for(tokio::time::Duration::from_secs(2)).await;

    // client.shutdown().await;

    // tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;

    println!("Websocket disconnected? {}", client.is_disconnected().await);
    client.wait_for_shutdown();

    println!("done")
}
