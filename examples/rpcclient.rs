use dcrdrs::{
    dcrutil::app_data,
    rpcclient::{client, connection, extensions_commands::Extension, notify},
};
use std::{fs, path::PathBuf};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
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

        ..Default::default()
    };

    let client = client::new(config, notif_handler).await.unwrap();
    // let ll = Arc::new(client);
    // let clone_cli = Arc::clone(ll.clone());
    //  task::Poll::is_ready(shutdown(&client));
    println!("waiting for shutdown!!!");

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
