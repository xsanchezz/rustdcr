use dcrdrs::{dcrutil::app_data, rpcclient};
use std::{fs, net, path::PathBuf, sync::Arc, task, thread, time};

#[tokio::main]
async fn main() {
    // Get dcrd app directory, if none is found use current path.
    let mut app_dir = match app_data::get_app_data_dir("dcrd".into(), false) {
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

    let client = rpcclient::client::new(config, notif_handler).await.unwrap();
    // let ll = Arc::new(client);
    // let clone_cli = Arc::clone(ll.clone());

    //  task::Poll::is_ready(shutdown(&client));
    println!("waiting for shutdown!!!");

    // let handle = &client.is_disconnected();

    // tokio::spawn(async move {
    //     tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;
    //     println!("is websocket disconnected, {}", handle.await);
    // });

    client.wait_for_shutdown();
}

fn block_connected() {
    println!("works");
}
