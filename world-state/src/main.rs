extern crate alloc;

use crate::{
  node_codec::ExtensionLayout, transaction::Transaction, simple_trie::SimpleTrie
};
use anyhow::{Result, anyhow, Error};
use appconfig::CONFIG;
use cron::Cron;
use db::{KVDB, KVDatabase};
use keccak_hasher::{keccak_256, KeccakHasher};
use kvdb::{KeyValueDB, DBValue};
use kvdb_rocksdb::{Database, DatabaseConfig};
use memory_db::{MemoryDB, HashKey};
use hash_db::{Hasher, AsHashDB, HashDB, HashDBRef, Prefix};
use metacontract::MetaContract;
use metadata::Metadata;
use rqlite::RQLite;
use transaction_receipt::TransactionReceipt;
use trie_db::{TrieDBMutBuilder, TrieMut, TrieDBNodeIterator, TrieDBBuilder, TrieLayout, node::{NodePlan, ValuePlan, Node, Value}, TrieDB, Trie, TrieDBMut};
use hex_literal::hex;
use rlp::{encode, decode, Decodable, Rlp, DecoderError};
use types::{TrieResult, DecodableEnum};
use serde_json::{Value as SerdeValue};
use serde::Serialize;

use std::{env, sync::Arc, collections::HashMap, ops::Deref, any::Any};

mod appconfig;
mod node_codec;
mod transaction;
mod transaction_receipt;
mod cron;
mod metadata;
mod metacontract;
mod simple_trie;
mod db;
mod types;
mod utils;
mod rqlite;

fn main() -> Result<()> {
  let args: Vec<String> = env::args().collect();
  let method = &args[1];

  // println!("method: {:?}", method);

  let result = match method.as_str() {
    "init" => init(),
    "insert_tx" => Transaction::insert_tx(),
    "get_pending_tx" => Transaction::get_pending_tx(),
    "filter_trie" => filter_trie(),
    "update_tx_status" => Transaction::update_tx_status(),
    "insert_trie" => {
      let trie_key =  args[2].as_str();
      let trie_value =  args[3].as_str();

      insert_trie(trie_key, trie_value)
    },
    "insert_trie_batch" => {
      let trie_key =  args[2].as_str();
      let trie_value =  args[3].as_str();

      insert_trie_batch(trie_key, trie_value)
    },
    _ => TrieResult { success: false, result: None },
  };
  // println!("result: {:?}", result.result.unwrap());
  println!("{:?}", serde_json::to_string(&result).unwrap_or("".to_string()));
  
  Ok(())
}

fn init() -> TrieResult {
  RQLite::create_tables();
  TrieResult { success: true, result: None }
}

fn insert_trie_batch(
  trie_key: &str,
  trie_value: &str,
) -> TrieResult {
  let mut success = false;
  // let mut result = None;

  let mut values: Vec<String> = Vec::new();

  match trie_key {
    "tx" => {
      let t: Vec<Transaction> = serde_json::from_str(trie_value).unwrap();
      values = t
              .into_iter()
              .map(|f| serde_json::to_string(&f).unwrap())
              .collect();
     
    },
    "cron" => {
      let t: Vec<Cron> = serde_json::from_str(trie_value).unwrap();
      values = t
        .into_iter()
        .map(|f| serde_json::to_string(&f).unwrap())
        .collect();
      
    },
    "receipt" => {
      let t: Vec<TransactionReceipt> = serde_json::from_str(trie_value).unwrap();
      values = t
        .into_iter()
        .map(|f| serde_json::to_string(&f).unwrap())
        .collect();
    },
    "metadata" => {
      let t: Vec<Metadata> = serde_json::from_str(trie_value).unwrap();
      values = t
        .into_iter()
        .map(|f| serde_json::to_string(&f).unwrap())
        .collect();
    },
    "metacontract" => {
      let t: Vec<MetaContract> = serde_json::from_str(trie_value).unwrap();
      values = t
        .into_iter()
        .map(|f| serde_json::to_string(&f).unwrap())
        .collect();
    },
    _ => {
      success = false;
    },
  };

  if values.len() > 0 {
    for val in values {
      insert_trie(trie_key, &val);
    }
  }

  TrieResult { 
    success, 
    result: Some(trie_value.to_string()), 
  }
}

fn insert_trie(
  trie_key: &str,
  trie_value: &str,
) -> TrieResult {
  println!("trie_key: {:?}, trie_value: {:?}", trie_key, trie_value);
  // let args: Vec<String> = env::args().collect();

  // let trie_key = args[2].as_str();
  // let trie_value = args[3].as_str();

  let db_path;
  let new_key;
  let new_value;

  let mut success = false;
  let mut result = None;

  // println!("trie_key: {:?}, trie_value: {:?}", trie_key, trie_value);

  match trie_key {
    "tx" => {
      db_path = CONFIG.get::<String>("TX_DB_PATH").unwrap();
      let p: Transaction = serde_json::from_str(trie_value).unwrap();
      new_key = Some(p.hash.as_bytes().to_vec());
      new_value = Some(encode(&p).to_vec());
    },
    "cron" => {
      db_path = CONFIG.get::<String>("CRON_DB_PATH").unwrap();
      let p: Cron = serde_json::from_str(trie_value).unwrap();
      new_key = Some(p.program_id.as_bytes().to_vec());
      new_value = Some(encode(&p).to_vec());
    },
    "receipt" => {
      db_path = CONFIG.get::<String>("TX_RECEIPT_DB_PATH").unwrap();
      let p: TransactionReceipt = serde_json::from_str(trie_value).unwrap();
      new_key = Some(p.hash.as_bytes().to_vec());
      new_value = Some(encode(&p).to_vec());
    },
    "metadata" => {
      db_path = CONFIG.get::<String>("METADATA_DB_PATH").unwrap();
      let p: Metadata = serde_json::from_str(trie_value).unwrap();
      new_key = Some(p.hash.as_bytes().to_vec());
      println!("new_key: {:?} {:?}", p.hash, hex::encode(new_key.clone().unwrap()));
      new_value = Some(encode(&p).to_vec());
    },
    "metacontract" => {
      db_path = CONFIG.get::<String>("METACONTRACT_DB_PATH").unwrap();
      let p: MetaContract = serde_json::from_str(trie_value).unwrap();
      new_key = Some(p.program_id.as_bytes().to_vec());
      new_value = Some(encode(&p).to_vec());
    },
    _ => {
      db_path = "".to_string();
      new_key = None;
      new_value = None;
    },
  };

  if new_key.is_some() {
    let pairs: Vec<(Vec<u8>, Vec<u8>)> = vec![
      (new_key.unwrap(), new_value.unwrap()),
    ];

    build_trie_db::<ExtensionLayout>(
      trie_key, 
      &db_path, 
      &pairs,
    );
    success = true;
    result = Some(trie_value.to_string());
  }

  TrieResult { 
    success, 
    result, 
  }
}

fn child_filter<T>(items: Result<T, DecoderError>, filter_array: Vec<SerdeValue>) -> Option<SerdeValue> 
where 
  T: Clone + Into<HashMap<String, SerdeValue>> + Serialize,
{
  match items {
    Ok(tx) => {

      let new_tx = tx.clone();
      let mut tx_map: HashMap<String, SerdeValue> = tx.into();

      let mut found = 0;
      for (index, item) in filter_array.iter().enumerate() {

        if let SerdeValue::Object(obj) = item {
          for (k, v) in obj {
            if tx_map.get(k) == Some(v) {
             found = found+1;
             if found == filter_array.len() {
                return Some(serde_json::to_value(&new_tx).unwrap());
             }  
            }
          }
        }
      }
    },
    _ => return None,
  }
  None
}
fn filter_trie() -> TrieResult {
  let args: Vec<String> = env::args().collect();

  let trie_key = args[2].as_str();
  let filter_key = args[3].clone();
  
  // let t = CONFIG.get("TX_DB_PATH").unwrap();

  let db_path = match trie_key {
    "tx" => CONFIG.get::<String>("TX_DB_PATH").unwrap(),
    "cron" => CONFIG.get::<String>("CRON_DB_PATH").unwrap(),
    "receipt" => CONFIG.get::<String>("TX_RECEIPT_DB_PATH").unwrap(),
    "metadata" => CONFIG.get::<String>("METADATA_DB_PATH").unwrap(),
    "metacontract" => CONFIG.get::<String>("METACONTRACT_DB_PATH").unwrap(),
    _ => "".to_string(),
  };

  let mut filters = "".to_string();
  
  if args.len() > 4 {
    filters = args[4].clone();
  }

  let trie_results = get_trie_results(trie_key, &db_path, Some(filter_key));

  // println!("filter_array: {:?}", filter_array);
  let mut new_results: Vec<SerdeValue> = Vec::new();
  for val in trie_results.iter() {
    // let mut dec_val: Box<dyn Decodable>;

    if let Some(dec_val) = DecodableEnum::decode(trie_key, &Rlp::new(&val)){
      match dec_val {
        DecodableEnum::Transaction(transaction) => {
          if filters == "".to_string() {
            new_results.push(serde_json::to_value(&transaction.unwrap()).unwrap());
          } else {
            let mut filter_array: Vec<SerdeValue> = serde_json::from_str(&filters).unwrap();
            let item = child_filter(transaction, filter_array.clone());
            if !item.is_none() {
              new_results.push(item.unwrap());
            }
          }
        },
        DecodableEnum::Cron(cron) => {
          if filters == "".to_string() {
            new_results.push(serde_json::to_value(&cron.unwrap()).unwrap());
          } else {
            let mut filter_array: Vec<SerdeValue> = serde_json::from_str(&filters).unwrap();
            let item = child_filter(cron, filter_array.clone());
            if !item.is_none() {
              new_results.push(item.unwrap());
            }
          }
        },
        DecodableEnum::Metadata(metadata) => {
          if filters == "".to_string() {
            new_results.push(serde_json::to_value(&metadata.unwrap()).unwrap());
          } else {
            let mut filter_array: Vec<SerdeValue> = serde_json::from_str(&filters).unwrap();
            let item = child_filter(metadata, filter_array.clone());
            if !item.is_none() {
              new_results.push(item.unwrap());
            }
          }
        },
        DecodableEnum::MetaContract(metacontract) => {
          if filters == "".to_string() {
            new_results.push(serde_json::to_value(&metacontract.unwrap()).unwrap());
          } else {
            let mut filter_array: Vec<SerdeValue> = serde_json::from_str(&filters).unwrap();
            let item = child_filter(metacontract, filter_array.clone());
            if !item.is_none() {
              new_results.push(item.unwrap());
            }
          }
        },
        DecodableEnum::Receipt(receipt) => {
          if filters == "".to_string() {
            new_results.push(serde_json::to_value(&receipt.unwrap()).unwrap());
          } else {
            let mut filter_array: Vec<SerdeValue> = serde_json::from_str(&filters).unwrap();
            let item = child_filter(receipt, filter_array.clone());
            if !item.is_none() {
              new_results.push(item.unwrap());
            }
          }
        },
        _ => (),
      }
    }
  }

  let mut success = false;
  let mut result = Some("".to_string());

  if new_results.len() > 0 {
    success = true;
    result = Some(serde_json::to_string(&new_results).unwrap_or("".to_string()));
  } else {
    result = Some("Record not found".to_string());
  }

  TrieResult { success, result }
}

fn get_trie_root(key: &str) -> [u8; 32] {
  let KVDatabase {db, ..} = KVDatabase::open(&CONFIG.get::<String>("ROOT_DB_PATH").unwrap());
  let root = db.get(0, key.as_bytes());

  let mut array = [0u8; 32];
  
  match root {
    Ok(val) => {
      match val {
          Some(value) => {
            array.copy_from_slice(&value[..32]);
            
            array
          },
          _ => array,
      }
      
    },
    _ => array,
  }
}

fn get_trie_results(root_key: &str, db_path: &str, prefix: Option<String> ) -> Vec<Vec<u8>> {
  let memdb = KVDatabase::open(db_path);

  let root = get_trie_root(root_key);

  let db = &memdb.as_hash_db();

  let trie = TrieDBBuilder::<ExtensionLayout>::new(db, &root).build();

  let iter = TrieDBNodeIterator::new(&trie);

  let mut results = Vec::new();

  match iter {
    Ok(mut it) => {
      if prefix.is_some() {
        it.prefix(&prefix.unwrap().as_bytes()).unwrap();
      }
    
    
      for node in it {
        // println!("node: {:?}", node);
        let node = node.map_err(|e| format!("TrieDB node iterator error: {}", e)).unwrap();
        match node.2.node_plan() {
          NodePlan::Leaf { value, .. } | NodePlan::NibbledBranch { value: Some(value), .. } => {
            if let ValuePlan::Inline(_) = value {
              // match node.1.to_owned()
              match node.2.node() {
                Node::Leaf(_, val) => {
                  match val {
                    Value::Inline(bytes) => {
                      // println!("val: {:?}", hex::encode(bytes));
                      results.push(bytes.to_vec());
                    }
                    Value::Node(_) => todo!(),
                  }
                },
                _ => (),
              }
            }
          },
          _ => (),
        }
      }
    },
    _ => (),
  }

  // for val in results.iter() {
    // println!("val: {:?}", hex::encode(val));
  // }

  results
}

fn build_trie_db<T: TrieLayout>(
  root_key: &str,
  db_path: &str,
	pairs: &[(Vec<u8>, Vec<u8>)],
) -> (Arc<dyn KeyValueDB>, <KeccakHasher as Hasher>::Out) {

  let KVDatabase {db: memdb, ..} = KVDatabase::open(db_path);

  let root_tx = get_trie_root(root_key);

  let (db, overlay, root) = {
    let mut overlay = HashMap::new();
    let mut root: <KeccakHasher as Hasher>::Out = Default::default();

    let mut trie = SimpleTrie::new(memdb, &mut overlay);
    {
      if root_tx == [0u8; 32] {
        root = Default::default();
  
        let mut trie_db = trie_db::TrieDBMutBuilder::<ExtensionLayout>::new(&mut trie, &mut root).build();
  
        for (x, y) in pairs.iter() {
          trie_db.insert(x, y).expect("trie insertion failed");
        }
        trie_db.commit();
      } else {
        root = root_tx;
        let mut trie_db = trie_db::TrieDBMutBuilder::<ExtensionLayout>::from_existing(&mut trie, &mut root).build();
  
          for (x, y) in pairs.iter() {
            println!("trie insert: {:?}", hex::encode(x));
            trie_db.insert(x, y).expect("trie insertion failed");
          }
          trie_db.commit();
      }
    }


    (trie.db, overlay, root)

  };

  let mut transaction = db.transaction();
  for (key, value) in overlay.into_iter() {
    match value {
      Some(value) => {
        transaction.put(0, &key[..], &value[..]);

        let stmt = format!("INSERT OR REPLACE INTO {} (trie_key, trie_value) VALUES ('{}', '{}')", 
          root_key,
          hex::encode(&key[..]),
          hex::encode(&value[..])
        );
        RQLite::execute(stmt.as_str());
      },
      None => {
        transaction.delete(0, &key[..]);

        let stmt = format!("DELETE FROM {} WHERE trie_key = '{}'", 
          root_key,
          hex::encode(&key[..]),
        );
        RQLite::execute(stmt.as_str());

      },
    }
  }
  db.write(transaction).expect("Failed to write transaction");

  let KVDatabase {db: root_db, ..} = KVDatabase::open(&CONFIG.get::<String>("ROOT_DB_PATH").unwrap());

  let mut root_tx = root_db.transaction();
  root_tx.put(0, root_key.as_bytes() ,&root);
  let root_result = root_db.write(root_tx);

  match root_result {
    Ok(_) => {
      let stmt = format!("INSERT OR REPLACE INTO roots (root_key, root_value) VALUES ('{}', '{}')", 
          root_key,
          hex::encode(&root)
        );
      RQLite::execute(stmt.as_str());
    },
    _ => (),
  }

  (db, root)
}

#[test]
fn test_iter() {
  let results = get_trie_results("tx", &CONFIG.get::<String>("TX_DB_PATH").unwrap(), None);

  //get status = 1
  let mut new_results = Vec::new();
  for val in results.iter() {
    let dec_tx = Transaction::decode(&Rlp::new(&val));

    match dec_tx {
      Ok(tx) => {
        if tx.status == 1 {
          new_results.push(tx);
        }
      },
      _ => (),
    }
  }

  for val in new_results.iter() {
    println!("status: {:?}", val);
  }
}
#[test]
fn test_get_trie() {
  let memdb = KVDatabase::open(&CONFIG.get::<String>("METADATA_DB_PATH").unwrap());

  let root = get_trie_root("metadata");

  let db = &memdb.as_hash_db();

  let trie = TrieDBBuilder::<ExtensionLayout>::new(db, &root).build();

  let result = trie.get(b"3kmAHv8M8zN8A3ofG1jygVmGeMohiRhdVBCDHwzoxJgH7TTGKXuhDL4XHeo2J2ZfKijhY4J8wYhPMHagzdUh6ZSQEXgsTeuKfMVALwuVLnW6jJp1cUXxd5uGXGuijA9UGstf0xee1f0084514f12e6f02557e43f76669d81ef0022").expect("key not found");
  println!("result: {:?}", hex::encode(result.unwrap()));
  
}

#[test]
fn test_simple_trie() {
  let tx = Transaction {
    hash: "1234".into(),
    method: "method".into(),
    program_id: "program_id".into(),
    data_key: "data_key".into(),
    data: "data".into(),
    public_key: "public_key".into(),
    alias: "alias".into(),
    timestamp: 0,
    chain_id: "001".to_string(),
    token_address: "12345".to_string(),
    token_id: "1".to_string(),
    version: "version".into(),
    mcdata: "".into(),
    status: 0,
  };

  let tx2 = Transaction {
    hash: "1235".into(),
    method: "method2".into(),
    program_id: "program_id2".into(),
    data_key: "data_key2".into(),
    data: "data2".into(),
    public_key: "public_key2".into(),
    alias: "alias2".into(),
    timestamp: 0,
    chain_id: "001".to_string(),
    token_address: "12345".to_string(),
    token_id: "1".to_string(),
    version: "version2".into(),
    mcdata: "".into(),
    status: 0,
  };

  let tx3 = Transaction {
    hash: "1236".into(),
    method: "method3".into(),
    program_id: "program_id2".into(),
    data_key: "data_key2".into(),
    data: "data2".into(),
    public_key: "public_key2".into(),
    alias: "alias2".into(),
    timestamp: 0,
    chain_id: "001".to_string(),
    token_address: "12345".to_string(),
    token_id: "1".to_string(),
    version: "version2".into(),
    mcdata: "".into(),
    status: 0,
  };

  let tx4 = Transaction {
    hash: "4236".into(),
    method: "method2".into(),
    program_id: "program_id2".into(),
    data_key: "data_key2".into(),
    data: "data2".into(),
    public_key: "public_key2".into(),
    alias: "alias2".into(),
    timestamp: 0,
    chain_id: "001".to_string(),
    token_address: "12345".to_string(),
    token_id: "1".to_string(),
    version: "version3".into(),
    mcdata: "".into(),
    status: 1,
  };
  
  let pairs: Vec<(Vec<u8>, Vec<u8>)> = vec![
    (tx.hash.as_bytes().to_vec(), encode(&tx).to_vec()),
    (tx2.hash.as_bytes().to_vec(), encode(&tx2).to_vec()),
    (tx3.hash.as_bytes().to_vec(), encode(&tx3).to_vec()),
    (tx4.hash.as_bytes().to_vec(), encode(&tx4).to_vec()),
  ];

  let enc_tx = encode(&tx4).to_vec();
  let dec_tx = Transaction::decode(&Rlp::new(&enc_tx)).unwrap();
  println!("status: {:?}", dec_tx.status);

  let (db, root) = build_trie_db::<ExtensionLayout>("tx", &CONFIG.get::<String>("TX_DB_PATH").unwrap(), &pairs);
  
}

#[test]
fn test_gen_trie() {
  // let mut db = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default();
  let KVDatabase {db, ..} = KVDatabase::open(&CONFIG.get::<String>("TX_TEST_DB_PATH").unwrap());
  let mut overlay: HashMap<Vec<u8>, Option<Vec<u8>>> = HashMap::new();
  let mut t = SimpleTrie::new(db, &mut overlay);

  let mut root: <KeccakHasher as Hasher>::Out = Default::default();

  let mut trie = TrieDBMutBuilder::<ExtensionLayout>::new(&mut t, &mut root).build();

  // trie.insert(&[0u8; 32], &[0u8;32]).expect("bla");
  trie.insert(b"3kmAHv8M8zN8A3ofG1jygVmGeMohiRhdVBCDHwzoxJgH7TTGKXuhDL4XHeo2J2ZfKijhY4J8wYhPMHagzdUh6ZSQEXgsTeuKfMVALwuVLnW6jJp1cUXxd5uGXGuijA9UGstf0xee1f0084514f12e6f02557e43f76669d81ef0022", b"value").expect("trie insertion failed");

  let result = trie.get(b"3kmAHv8M8zN8A3ofG1jygVmGeMohiRhdVBCDHwzoxJgH7TTGKXuhDL4XHeo2J2ZfKijhY4J8wYhPMHagzdUh6ZSQEXgsTeuKfMVALwuVLnW6jJp1cUXxd5uGXGuijA9UGstf0xee1f0084514f12e6f02557e43f76669d81ef0022").expect("trie get failed");

  println!("result: {:?}", hex::encode(result.unwrap()));
}

#[test]
fn test_remove_root() {
  let KVDatabase {db, ..} = KVDatabase::open(&CONFIG.get::<String>("ROOT_DB_PATH").unwrap());
  let mut root_tx = db.transaction();
  root_tx.delete(0, CONFIG.get::<String>("METADATA_KEY").unwrap().as_bytes());
  db.write(root_tx).expect("Failed to write transaction");
}

#[test]
fn test_get_trie_root() {
  let root = get_trie_root(&CONFIG.get::<String>("METADATA_KEY").unwrap());

  println!("root: {:?} {:?}", hex::encode(root), [0u8;32]);
}