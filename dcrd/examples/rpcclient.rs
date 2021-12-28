use rustdcr::{
    dcrjson::types::TxRawResult,
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

        on_tx_accepted_verbose: Some(|tx: TxRawResult| {
            println!("Transaction Accepted\n-TX {:?}", tx)
        }),

        ..Default::default()
    };

    let mut client = client::new(config, notif_handler).await.unwrap();

    client
        .notify_new_transactions(true)
        .await
        .expect("could not sent notify new tx notif to server")
        .await
        .expect("server sent an error on notify new tx");

    client
        .notify_blocks()
        .await
        .expect("unable to send block notification command to server")
        .await
        .expect("server replied with an error on notify blocks");

    tokio::signal::ctrl_c().await.unwrap();
    client.shutdown().await;
}
