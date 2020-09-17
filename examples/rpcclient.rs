use dcrdrs::{dcrutil::app_data, rpcclient};
use std::{fs::read, path::PathBuf};

fn main() {
    let mut app_dir = match app_data::get_app_data_dir(&mut "dcrd".into(), false) {
        Some(dir) => dir,

        None => PathBuf::new().join("."),
    };

    println!("connecting");
    app_dir.push("rpc.cert");

    println!("{}", app_dir.to_str().unwrap());

    let certs = std::fs::read_to_string(app_dir).unwrap();

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

    let _client = rpcclient::client::new(config, notif_handler).unwrap();
    // String::from_utf8_lossy(read(app_dir).unwrap());
    // dcrdrs::rpcclient::client::new(mut config, notif_handler)
}

fn block_connected() {
    println!("works");
}
