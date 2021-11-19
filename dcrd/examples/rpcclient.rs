use rustdcr::{
    chaincfg::chainhash::Hash,
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
                "\t\t\t\tBlock Connected Notif\n-Block header: {:?} \n-Transactions: {:?}",
                block_header, transactions,
            )
        }),

        on_block_disconnected: Some(|block_header: Vec<u8>| {
            println!(
                "\t\t\t\tBlock Disconnected Notif\n-Block header: {:?}",
                block_header,
            )
        }),

        on_work: Some(|data: Vec<u8>, target: Vec<u8>, reason: String| {
            println!(
                "\t\t\t\tOn Work Notif\n-Data: {:?}\n-Transactions: {:?}\n-Reason: {:?}",
                data, target, reason,
            )
        }),

        on_new_tickets: Some(
            |hash: Hash, height: i64, stake_diff: i64, tickets: Vec<Hash>| {
                println!(
                    "\t\t\t\tOn Tickets Notif\n-Hash: {:?}\n-Height: {:?}\n-Stake Diff: {:?}\n-Tickets: {:?}",
                    hash.string().unwrap(),height, stake_diff,tickets,
                )
            },
        ),

        on_tx_accepted: Some(|hash: Hash, amount: dcrutil::amount::Amount| {
            println!(
                "\t\t\t\tOn TX Accepted\n-Hash: {:?}\n-Amount:{:?}",
                hash.string(),
                amount.to_string()
            )
        }),

        on_tx_accepted_verbose: Some(|tx_details: rustdcr::dcrjson::types::TxRawResult| {
            println!(
                "\t\t\t\tOn TX Accepted Verbose\n-TX Raw Result: {:?}",
                tx_details
            )
        }),

        on_stake_difficulty: Some(|hash: Hash, height: i64, stake_diff: i64| {
            println!(
                "\t\t\t\tOn Stake Difficulty\n-Hash: {:?} \n-Height: {} \n-Stake Difference: {}",
                hash.string().unwrap(),
                height,
                stake_diff,
            )
        }),

        on_reorganization: Some(
            |old_hash: Hash, old_height: i32, new_hash: Hash, new_height: i32| {
                println!(
                    "\t\t\t\tOn Block Reorganization
                    \n-Old Hash: {:?} \n-Old Height: {:?} \n-New Hash: {:?} \n-New Height: {:?}",
                    old_hash.string().unwrap(),
                    old_height,
                    new_hash.string().unwrap(),
                    new_height
                )
            },
        ),

        ..Default::default()
    };

    let mut client = client::new(config, notif_handler).await.unwrap();

    client
        .notify_new_transactions(true)
        .await
        .expect("Could not sent notify new tx notif to server")
        .await
        .expect("Server sent an error on notify new tx");

    client
        .notify_blocks()
        .await
        .expect("Unable to send block notification command to server")
        .await
        .expect("Server replied with an error on notify blocks");

    client
        .notify_stake_difficulty()
        .await
        .expect("Unable to send stake notification command to server")
        .await
        .expect("Server replied with an error on stake notification");

    client.wait_for_shutdown().await;
}
