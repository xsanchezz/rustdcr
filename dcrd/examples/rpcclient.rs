use rustdcr::{
    dcrutil,
    rpcclient::{client, connection, notify},
};

use std::{fs, path::PathBuf};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    // Get dcrd app directory, if none is found use current path.
    let mut app_dir = match dcrutil::get_app_data_dir("dcrd".into(), false) {
        Some(dir) => dir,

        None => PathBuf::new().join("."),
    };

    if !app_dir.exists() {
        panic!("Path does not exist")
    }

    app_dir.push("rpc.cert");

    let certs = fs::read_to_string(app_dir).unwrap();

    let config = connection::ConnConfig {
        host: "127.0.0.1:19109".to_string(),
        password: "rpcpassword".to_string(),
        user: "rpcuser".to_string(),
        certificates: certs,

        ..Default::default()
    };

    let notif_handler = notify::NotificationHandlers {
        on_client_connected: Some(|| {
            println!("client connected");
        }),

        on_block_connected: Some(|block_header: Vec<u8>, transactions: Vec<Vec<u8>>| {
            println!(
                "Block Connected Notif\n- Block header: {:?} \n-Transactions: {:?}",
                block_header, transactions,
            )
        }),

        on_block_disconnected: Some(|block_header: Vec<u8>| {
            println!(
                "Block Disconnected Notif\n- Block header: {:?}",
                block_header,
            )
        }),

        ..Default::default()
    };

    let mut client = client::new(config, notif_handler).await.unwrap();

    client
        .notify_blocks()
        .await
        .expect("unable to send block notification command to server")
        .await
        .expect("server replied with an error on notify blocks");

    // Send request to RPC server
    let block_count_future = client.get_block_count().await.unwrap();

    // Wait for response in an async thread
    tokio::spawn(async move {
        let block_count = block_count_future.await.unwrap();

        println!("Current synced block count is {:?}", block_count);
    });

    tokio::signal::ctrl_c().await.unwrap();
    client.shutdown().await;
}
