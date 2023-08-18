use std::collections::HashMap;

use rlp_derive::{RlpEncodable, RlpDecodable};
use serde::{Serialize, Deserialize};
use serde_json::{Value, Number};

#[derive(Serialize, Deserialize, RlpEncodable, RlpDecodable, Debug, Clone)]
pub struct TransactionReceipt {
    pub hash: String,
    pub program_id: String,
    pub status: u64,
    pub timestamp: u64,
    pub error_text: String,
    pub data: String,
}

impl From<TransactionReceipt> for HashMap<String, Value> {
  fn from(r: TransactionReceipt) -> Self {
    let mut map = HashMap::new();
    map.insert("hash".to_string(), Value::String(r.hash));
    map.insert("program_id".to_string(), Value::String(r.program_id));
    map.insert("error_text".to_string(), Value::String(r.error_text));
    map.insert("data".to_string(), Value::String(r.data));
    map.insert("timestamp".to_string(), Value::Number(Number::from(r.timestamp)));
    map.insert("status".to_string(), Value::Number(Number::from(r.status)));

    map
  }
}