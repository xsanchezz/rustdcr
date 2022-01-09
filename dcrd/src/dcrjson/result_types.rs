//! Houses all JSON result types.

use {log::warn, std::collections::HashMap};

/// Implements JSON RPC request structure to server.
#[derive(serde::Serialize)]
pub(crate) struct JsonRequest<'a> {
    pub jsonrpc: &'a str,
    pub method: &'a str,
    pub id: u64,
    pub params: &'a [serde_json::Value],
}

/// Implements JSON RPC response structure from server.
#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(default)]
pub struct JsonResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: serde_json::Value,
    pub result: serde_json::Value,
    pub params: Vec<serde_json::Value>,
    pub error: serde_json::Value,
}

/// Error returned by server.
#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(default)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

/// Provides an overview of an agenda in a consensus deployment.
#[derive(serde::Deserialize, Default, Debug)]
#[serde(default)]
pub struct AgendaInfo {
    pub status: String,
    pub since: i64,
    #[serde(rename = "starttime")]
    pub start_time: u64,
    #[serde(rename = "expiretime")]
    pub expire_time: u64,
}

/// GetBlockVerboseResult models the data from the getblock command when the
/// verbose flag is set.  When the verbose flag is not set, getblock returns a
/// hex-encoded string.  Contains Decred additions.
#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(default)]
pub struct GetBlockVerboseResult {
    pub hash: String,
    pub confirmations: i64,
    pub size: i32,
    pub height: i64,
    pub version: i32,
    #[serde(rename = "merkleroot")]
    pub merkle_root: String,
    #[serde(rename = "stakeroot")]
    pub stake_root: String,
    pub tx: Vec<String>,
    pub rawtx: Vec<TxRawResult>,
    #[serde(rename = "stx")]
    pub stake_tx: Vec<String>,
    #[serde(rename = "rawstx")]
    pub raw_stake_tx: Vec<TxRawResult>,
    pub time: i64,
    #[serde(rename = "mediantime")]
    pub median_time: i64,
    pub nonce: u32,
    #[serde(rename = "votebits")]
    pub vote_bits: u16,
    #[serde(rename = "finalstate")]
    pub final_state: String,
    pub voters: u16,
    #[serde(rename = "freshstake")]
    pub fresh_stake: u8,
    pub revocations: u8,
    #[serde(rename = "poolsize")]
    pub pool_size: u32,
    pub bits: String,
    #[serde(rename = "sbits")]
    pub stake_bits: f64,
    #[serde(rename = "extradata")]
    pub extra_data: String,
    #[serde(rename = "stakeversion")]
    pub stake_version: u32,
    pub difficulty: f64,
    #[serde(rename = "chainwork")]
    pub chain_work: String,
    #[serde(rename = "previoushash")]
    pub previous_hash: String,
    #[serde(rename = "nextblockhash")]
    pub next_block_hash: String,
}

/// BlockchainInfo models the data returned from the get_blockchain_info command.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: i64,
    pub headers: i64,
    #[serde(rename = "syncheight")]
    pub sync_height: i64,
    #[serde(rename = "bestblockhash")]
    pub best_block_hash: String,
    pub difficulty: u32,
    #[serde(rename = "difficultyratio")]
    pub difficulty_ratio: f64,
    #[serde(rename = "verificationprogress")]
    pub verification_progress: f64,
    #[serde(rename = "chainwork")]
    pub chain_work: String,
    #[serde(rename = "initialblockdownload")]
    pub initial_block_download: bool,
    #[serde(rename = "maxblocksize")]
    pub max_block_size: i64,
    pub deployments: HashMap<String, AgendaInfo>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone)]
#[serde(default)]
pub struct ScriptPubKeyResult {
    pub asm: String,
    pub hex: String,
    #[serde(rename = "reqSigs")]
    pub req_sigs: i32,
    #[serde(rename = "type")]
    pub script_type: String,
    pub addresses: Vec<String>,
    #[serde(rename = "commitamt")]
    pub commit_amount: f64,
}

/// ScriptSig models a signature script.  It is defined separately since it only
/// applies to non-coinbase.  Therefore the field in the Vin structure needs
/// to be a pointer.
#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone)]
#[serde(default)]
pub struct ScriptSig {
    pub asm: String,
    pub hex: String,
}

// TransactionInput represents the inputs to a transaction.  Specifically a
// transaction hash and output number pair. Contains Decred additions.
#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone)]
#[serde(default)]
pub struct TransactionInput {
    pub amount: f64,
    pub txid: String,
    pub vout: u32,
    pub tree: i8,
}

// TxRawDecodeResult models the data from the decoderawtransaction command.
#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone)]
#[serde(default)]
pub struct TxRawDecodeResult {
    pub txid: String,
    pub version: i32,
    pub locktime: u32,
    pub expiry: u32,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone)]
#[serde(default)]
pub struct EstimateSmartFeeResult {
    pub feerate: f64,
    pub errors: Vec<String>,
    pub blocks: i64,
}

/// TxRawResult models the data from the getrawtransaction command.
#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone)]
#[serde(default)]
pub struct TxRawResult {
    pub hex: Option<String>,
    #[serde(rename = "txid")]
    pub tx_id: Option<String>,
    pub version: i32,
    #[serde(rename = "locktime")]
    pub lock_time: u32,
    pub expiry: u32,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    #[serde(rename = "blockhash")]
    pub block_hash: String,
    #[serde(rename = "blockheight")]
    pub block_height: i64,
    #[serde(rename = "blockindex")]
    pub block_index: u32,
    pub confirmations: i64,
    pub time: i64,
    pub blocktime: i64,
}

/// Vin models parts of the tx data. It is defined separately since getrawtransaction, decoderawtransaction, and searchrawtransaction use the same structure.
#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone)]
#[serde(default)]
pub struct Vin {
    pub coinbase: String,
    pub stakebase: String,
    #[serde(rename = "txid")]
    pub tx_id: String,
    pub vout: u32,
    pub tree: i8,
    pub sequence: u32,
    #[serde(rename = "amountin")]
    pub amount_in: f64,
    #[serde(rename = "blockheight")]
    pub block_height: u32,
    #[serde(rename = "blockindex")]
    pub block_index: u32,
    #[serde(rename = "scriptSig")]
    pub script_sig: Option<ScriptSig>,
}

/// Vout models parts of the tx data.  It is defined separately since both
/// getrawtransaction and decoderawtransaction use the same structure.
#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone)]
#[serde(default)]
pub struct Vout {
    pub value: f64,
    pub n: u32,
    pub version: u16,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: ScriptPubKeyResult,
}

impl Vin {
    /// Returns a bool to show if a Vin is a Coinbase one or not.
    pub fn is_coin_base(&self) -> bool {
        !self.coinbase.is_empty()
    }

    /// Returns a bool to show if a Vin is a StakeBase one or not.
    pub fn is_stake_base(&self) -> bool {
        !self.stakebase.is_empty()
    }

    /// Provides a custom Marshal method for Vin.
    pub fn marshal_json(&self) -> Result<Vec<u8>, super::RpcServerError> {
        #[derive(serde::Serialize)]
        pub struct CoinbaseStruct {
            pub amountin: f64,
            pub blockheight: u32,
            pub blockindex: u32,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub coinbase: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub stakebase: Option<String>,
            pub sequence: u32,
        }

        let mut coin_or_stake_value = CoinbaseStruct {
            amountin: self.amount_in,
            blockheight: self.block_height,
            blockindex: self.block_index,
            coinbase: None,
            stakebase: None,
            sequence: self.sequence,
        };

        if self.is_coin_base() {
            coin_or_stake_value.coinbase = Some(self.coinbase.clone());

            match serde_json::to_vec(&coin_or_stake_value) {
                Ok(e) => return Ok(e),

                Err(e) => {
                    warn!("Error marshalling coinbase value, error: {}", e);
                    return Err(super::RpcServerError::Marshaller(e));
                }
            }
        }

        if self.is_stake_base() {
            coin_or_stake_value.stakebase = Some(self.stakebase.clone());

            match serde_json::to_vec(&coin_or_stake_value) {
                Ok(e) => return Ok(e),

                Err(e) => {
                    warn!("Error marshalling stakebase value, error: {}", e);
                    return Err(super::RpcServerError::Marshaller(e));
                }
            }
        }

        #[derive(serde::Serialize, Default)]
        pub struct Tx {
            pub txid: String,
            pub vout: u32,
            pub tree: i8,
            pub sequence: u32,
            pub amountin: f64,
            pub blockheight: u32,
            pub blockindex: u32,
        }

        let tx = Tx {
            txid: self.tx_id.clone(),
            vout: self.vout,
            tree: self.tree,
            sequence: self.sequence,
            amountin: self.amount_in,
            blockheight: self.block_height,
            blockindex: self.block_index,
        };

        match serde_json::to_vec(&tx) {
            Ok(e) => Ok(e),

            Err(e) => {
                warn!("Error marshalling stakebase value, error: {}", e);
                Err(super::RpcServerError::Marshaller(e))
            }
        }
    }
}
