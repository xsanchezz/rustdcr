use dcrdrs::{
    dcrutil::app_data,
    rpcclient::{client, connection, notify},
};
use std::{fs, path::PathBuf};

#[tokio::main]
async fn main() {
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
                "\n\n\nBlock header: {:?} \n\nTransactions: {:?}\n\n\n",
                block_header, transactions,
            )
        }),

        ..Default::default()
    };

    let mut client = client::new(config, notif_handler).await.unwrap();

    client.notify_blocks().await.unwrap();

    client.wait_for_shutdown();
}
