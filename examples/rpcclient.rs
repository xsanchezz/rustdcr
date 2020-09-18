use dcrdrs::{dcrutil::app_data, rpcclient};
use std::{fs, future, path::PathBuf, thread, time};

#[tokio::main]
async fn main() {
    // Get dcrd app directory, if none is found use current path.
    let mut app_dir = match app_data::get_app_data_dir(&mut "dcrd".into(), false) {
        Some(dir) => dir,

        None => PathBuf::new().join("."),
    };

    app_dir.push("rpc.cert");

    let certs = fs::read_to_string(app_dir).unwrap();

    let config = rpcclient::connection::ConnConfig {
        certificates: certs,
        password: "admin".to_string(),
        user: "admin".to_string(),
        ..Default::default()
    };

    let notif_handler = rpcclient::notify::NotificationHandlers {
        on_client_connected: Some(block_connected),
        ..Default::default()
    };

    let client = rpcclient::client::new(config, notif_handler).unwrap();

    shutdown(&client).await;

    client.wait_for_shutdown();
}

fn block_connected() {
    println!("works");
}

async fn shutdown(client: &rpcclient::client::Client) {
    println!(" down");
    thread::sleep(time::Duration::from_secs(10));
    println!("shutting down");
    client.shutdown();
    println!("server shut down");
}
