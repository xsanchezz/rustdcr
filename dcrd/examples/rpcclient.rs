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

        on_tx_accepted_verbose: Some(
            |tx_details: rustdcr::dcrjson::chain_command_result::TxRawResult| {
                println!(
                    "\t\t\t\tOn TX Accepted Verbose\n-TX Raw Result: {:?}",
                    tx_details
                )
            },
        ),

        on_stake_difficulty: Some(|hash: Hash, height: i64, stake_diff: i64| {
            println!(
                "\t\t\t\tOn Stake Difficulty\n-Hash: {:?} \n-Height: {} \n-Stake Difference: {}",
                hash.string().unwrap(),
                height,
                stake_diff,
            )
        }),

        ..Default::default()
    };

    let mut client = client::new(config, notif_handler).await.unwrap();

    let blk_count = client
        .get_block_count()
        .await
        .expect("Could not send get block count request to server");

    let blk_hash = client
        .get_block_hash(0)
        .await
        .expect("Could not send get block hash request to server");

    let blk_count_result = blk_count.await.unwrap();
    println!("Block Count Result: {}", blk_count_result);

    let blk_hash_result = blk_hash.await.unwrap();
    println!(
        "First/Zeroth Block Hash: {}",
        blk_hash_result.string().unwrap()
    );

    client
        .notify_new_transactions(false)
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

    client.wait_for_shutdown();
}
