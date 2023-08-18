use std::collections::HashMap;

use rlp_derive::{RlpEncodable, RlpDecodable};
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, RlpEncodable, RlpDecodable, Debug, Clone)]
pub struct MetaContract {
  pub program_id: String,
  pub public_key: String,
  pub cid: String,
}

impl From<MetaContract> for HashMap<String, Value> {
  fn from(m: MetaContract) -> Self {
    let mut map = HashMap::new();
    map.insert("program_id".to_string(), Value::String(m.program_id));
    map.insert("cid".to_string(), Value::String(m.cid));
    map.insert("public_key".to_string(), Value::String(m.public_key));

    map
  }
}