use std::{env, collections::HashMap, time::{SystemTime, UNIX_EPOCH}};

use rlp::{encode, Decodable, Rlp};
use rlp_derive::{RlpEncodable, RlpDecodable};
use serde::{Serialize, Deserialize};
use serde_json::{Value, Number};

use crate::{
  types::TrieResult, 
  node_codec::ExtensionLayout, 
  build_trie_db, 
  get_trie_results, appconfig::CONFIG, transaction_receipt::TransactionReceipt};

#[derive(Serialize, Deserialize, RlpEncodable, RlpDecodable, Debug, Clone)]
pub struct Transaction {
    pub hash: String,
    pub method: String,
    pub program_id: String,
    pub data_key: String,
    pub data: String,
    pub public_key: String,
    pub alias: String,
    pub timestamp: u64,
    pub chain_id: String,
    pub token_address: String,
    pub token_id: String,
    pub version: String,
    pub mcdata: String,
    pub status: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrieTransaction {
  pub transaction: Transaction,
  pub receipt: Option<TransactionReceipt>,
}

impl Transaction {
  pub fn insert_tx() -> TrieResult {
    // println!("insert tx");
    let args: Vec<String> = env::args().collect();
    let mut success = false;
    let mut result = None;
  
    let serde_tx: Result<TrieTransaction, _> = serde_json::from_str(args[2].clone().as_str());
  
    match serde_tx {
      Ok(tx) => {
        let pairs: Vec<(Vec<u8>, Vec<u8>)> = vec![
          (tx.transaction.hash.as_bytes().to_vec(), encode(&tx.transaction).to_vec()),
        ];
        // println!("tries: {:?}", tx);
        build_trie_db::<ExtensionLayout>(
          &CONFIG.get::<String>("TX_KEY").unwrap(), 
          &CONFIG.get::<String>("TX_DB_PATH").unwrap(), 
          &pairs
        );

        if tx.receipt.is_some() {
          let receipt = tx.receipt.unwrap();
          let receipt_pairs = vec![
            (receipt.hash.as_bytes().to_vec(), encode(&receipt).to_vec()),
          ];

          build_trie_db::<ExtensionLayout>(
            &CONFIG.get::<String>("TX_RECEIPT_KEY").unwrap(), 
            &CONFIG.get::<String>("TX_RECEIPT_DB_PATH").unwrap(), 
            &receipt_pairs
          );
        }
  
        success = true;
        result = Some(serde_json::to_string(&tx.transaction).unwrap_or("".to_string()));
      },
      _ => {
        result = Some("Error decoding transaction".to_string());
      }
    }
  
    TrieResult { 
      success, 
      result,
    }
  }

  pub fn get_pending_tx() -> TrieResult {
    let mut success = false;
    let mut result = None;
  
    let trie_results = get_trie_results(&CONFIG.get::<String>("TX_KEY").unwrap(), &CONFIG.get::<String>("TX_DB_PATH").unwrap(), None);
  
    let mut new_results = Vec::new();
    for val in trie_results.iter() {
      let dec_tx = Transaction::decode(&Rlp::new(&val));
  
      match dec_tx {
        Ok(tx) => {
          if tx.status == 0 {
            new_results.push(tx);
          }
        },
        _ => (),
      }
    }
  
    success = true;
    result = Some(serde_json::to_string(&new_results).unwrap_or("".to_string()));
  
    TrieResult { 
      success, 
      result, 
    }
  }

  pub fn update_tx_status() -> TrieResult {
    let args: Vec<String> = env::args().collect();
    let mut success = false;
    let mut result = None;

    let trie_key = args[2].clone();
    let status = args[3].clone().parse::<u64>().unwrap();

    let trie_results = get_trie_results(
      &CONFIG.get::<String>("TX_KEY").unwrap(), 
      &CONFIG.get::<String>("TX_DB_PATH").unwrap(), 
      Some(trie_key),
    );

    if trie_results.len() > 0 {
      if let Some(val) = trie_results.get(0) {
        if let Ok(mut dec_tx) = Transaction::decode(&Rlp::new(&val)) {
          dec_tx.status = status;

          let pairs: Vec<(Vec<u8>, Vec<u8>)> = vec![
            (dec_tx.hash.as_bytes().to_vec(), encode(&dec_tx).to_vec()),
          ];

          build_trie_db::<ExtensionLayout>(
            &CONFIG.get::<String>("TX_KEY").unwrap(), 
            &CONFIG.get::<String>("TX_DB_PATH").unwrap(), 
            &pairs
          );

          let mut error_text = "".to_string();
  
          if args.len() > 4 {
            error_text = args[4].clone();
          }

          let now = SystemTime::now();
          let timestamp = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

          let receipt = TransactionReceipt {
            hash: dec_tx.hash.clone(),
            program_id: dec_tx.program_id.clone(),
            status,
            timestamp: timestamp.as_millis() as u64,
            error_text,
            data: dec_tx.data.clone(),
          };

          let receipt_pairs: Vec<(Vec<u8>, Vec<u8>)> = vec![
            (dec_tx.hash.as_bytes().to_vec(), encode(&receipt).to_vec()),
          ];

          build_trie_db::<ExtensionLayout>(
            &CONFIG.get::<String>("TX_RECEIPT_KEY").unwrap(), 
            &CONFIG.get::<String>("TX_RECEIPT_DB_PATH").unwrap(), 
            &receipt_pairs
          );

          success = true;
          result = Some(serde_json::to_string(&dec_tx).unwrap_or("".to_string()));
        }
      }
    } else {
      result = Some("Record not found".to_string());
    }

    TrieResult { 
      success, 
      result, 
    }
  }

}

impl From<Transaction> for HashMap<String, Value> {
  fn from(tx: Transaction) -> Self {
    let mut map = HashMap::new();
    map.insert("hash".to_string(), Value::String(tx.hash));
    map.insert("method".to_string(), Value::String(tx.method));
    map.insert("program_id".to_string(), Value::String(tx.program_id));
    // map.insert("data_key".to_string(), Value::String(tx.data_key));
    map.insert("data".to_string(), Value::String(tx.data));
    map.insert("public_key".to_string(), Value::String(tx.public_key));
    map.insert("alias".to_string(), Value::String(tx.alias));
    map.insert("timestamp".to_string(), Value::Number(Number::from(tx.timestamp)));
    map.insert("chain_id".to_string(), Value::String(tx.chain_id));
    map.insert("token_address".to_string(), Value::String(tx.token_address));
    map.insert("token_id".to_string(), Value::String(tx.token_id));
    map.insert("version".to_string(), Value::String(tx.version));
    map.insert("mcdata".to_string(), Value::String(tx.mcdata));
    map.insert("status".to_string(), Value::Number(Number::from(tx.status)));

    map
  }
}