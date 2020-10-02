use rustdcr::{
    chaincfg::chainhash::Hash,
    dcrutil::app_data,
    rpcclient::{client, connection, notify},
};

use std::{fs, path::PathBuf};

use slog;
use slog_scope;
use slog_stdlog;
use slog_term;

use std::fs::OpenOptions;

use slog::Drain;

#[tokio::main]
async fn main() {
    let log_path = "rpcclient.log";
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
        host: "127.0.0.1:19109".to_string(),
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
                "\n\n\n\t\t\t\tBlock Connected Notif\nBlock header: {:?} \n\nTransactions: {:?}\n\n\n",
                block_header, transactions,
            )
        }),

        on_block_disconnected: Some(|block_header: Vec<u8>| {
            println!(
                "\n\n\t\t\t\tBlock Disconnected Notif\n\nBlock header: {:?}\n\n\n",
                block_header,
            )
        }),

        on_work: Some(|data: Vec<u8>, target: Vec<u8>, reason: String| {
            println!(
                "\n\n\t\t\t\tOn Work Notif\n\nData: {:?}\n\n\nTransactions: {:?}\n\n\n\nReason: {:?}\n\n\n",
                data,target, reason,
            )
        }),

        on_new_tickets: Some(
            |hash: Hash, height: i64, stake_diff: i64, tickets: Vec<Hash>| {
                println!(
                "\n\n\t\t\t\tOn Tickets Notif\n\n\nHash: {:?}\n\n\nHeight: {:?}\n\n\nStake Diff: {:?}\n\n\nTickets: {:?}\n\n\n\n",
                hash.string().unwrap(),height, stake_diff,tickets,
            )
            },
        ),

        ..Default::default()
    };

    let mut client = client::new(config, notif_handler).await.unwrap();
    //    client.notify_work().await.unwrap();
    //   client.notify_new_tickets().await.unwrap();
    //  client.notify_blocks().await.unwrap();

    let blk_info = client.get_blockchain_info().await.unwrap();

    // Blockchain info is sent to a different async thread.
    tokio::spawn(async move {
        let a = blk_info.await.unwrap();

        println!(
            "\n\n\n\nBest Block Hash {} \n\nBlocks {}  \n\nChain {} \n\n Chain Work {} \n\n Deployments {:?} \n\n Difficulty {} \n\n Difficulty Ratio {} \n\n Headers {} \n\n Initial Block Download {} \n\n Max Block Size {} \n\n Sync Height {} \n\n Verification Progress {}",
            a.best_block_hash,
            a.blocks,
            a.chain,
            a.chain_work,
            a.deployments,
            a.difficulty,
            a.difficulty_ratio,
            a.headers,
            a.initial_block_download,
            a.max_block_size,
            a.sync_height,
            a.verification_progress
        )
    });

    client.wait_for_shutdown();
}
