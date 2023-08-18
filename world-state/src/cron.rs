use std::collections::HashMap;

use rlp_derive::{RlpEncodable, RlpDecodable};
use serde::{Serialize, Deserialize};
use serde_json::{Value, Number};

#[derive(Serialize, Deserialize, RlpEncodable, RlpDecodable, Debug, Clone)]
pub struct Cron {
  pub program_id: String,
  pub public_key: String,
  pub cid: String,
  pub epoch: u64,
  pub status: u64,
}

impl From<Cron> for HashMap<String, Value> {
  fn from(cron: Cron) -> Self {
    let mut map = HashMap::new();
    map.insert("program_id".to_string(), Value::String(cron.program_id));
    map.insert("public_key".to_string(), Value::String(cron.public_key));
    map.insert("cid".to_string(), Value::String(cron.cid));
    map.insert("epoch".to_string(), Value::Number(Number::from(cron.epoch)));
    map.insert("status".to_string(), Value::Number(Number::from(cron.status)));

    map
  }
}