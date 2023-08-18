use std::collections::HashMap;

use rlp_derive::{RlpEncodable, RlpDecodable};
use serde::{Serialize, Deserialize};
use serde_json::{Value, Number};

#[derive(Serialize, Deserialize, RlpEncodable, RlpDecodable, Debug, Clone)]
pub struct Metadata {
    pub hash: String,
    pub data_key: String,
    pub program_id: String,
    pub alias: String,
    pub chain_id: String,
    pub token_address: String,
    pub token_id: String,
    pub version: String,
    pub cid: String,
    pub public_key: String,
    pub loose: u64,
}

impl From<Metadata> for HashMap<String, Value> {
  fn from(m: Metadata) -> Self {
    let mut map = HashMap::new();
    map.insert("hash".to_string(), Value::String(m.hash));
    map.insert("data_key".to_string(), Value::String(m.data_key));
    map.insert("program_id".to_string(), Value::String(m.program_id));
    map.insert("alias".to_string(), Value::String(m.alias));
    map.insert("chain_id".to_string(), Value::String(m.chain_id));
    map.insert("token_address".to_string(), Value::String(m.token_address));
    map.insert("token_id".to_string(), Value::String(m.token_id));
    map.insert("version".to_string(), Value::String(m.version));
    map.insert("cid".to_string(), Value::String(m.cid));
    map.insert("public_key".to_string(), Value::String(m.public_key));
    map.insert("loose".to_string(), Value::Number(Number::from(m.loose)));

    map
  }
}