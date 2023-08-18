use std::{sync::Arc};

use hash_db::{HashDB, AsHashDB, Hasher, Prefix, HashDBRef};
use keccak_hasher::KeccakHasher;
use kvdb::KeyValueDB;
use kvdb_rocksdb::{DatabaseConfig, Database};
use trie_db::DBValue;

pub type KVDB = Arc<dyn KeyValueDB>;

// #[derive(Debug)]
pub struct KVDatabase {
  pub db: KVDB,
  hashed_null_node: <KeccakHasher as Hasher>::Out,
	null_node_data: Vec<u8>,
}

impl KVDatabase {
  pub fn open(db_path: &str) -> Self {
    let cfg = DatabaseConfig::with_columns(1);
    let db = Database::open(&cfg, db_path).expect("rocksdb works");
    KVDatabase {
      db: Arc::new(db),
      hashed_null_node: KeccakHasher::hash(&[0u8]),
      null_node_data: [0u8][..].into(),
    }
  }
}

impl<'a> AsHashDB<KeccakHasher, DBValue> for KVDatabase {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> {
		self
	}

	fn as_hash_db_mut<'b>(&'b mut self) -> &'b mut (dyn HashDB<KeccakHasher, DBValue> + 'b) {
		&mut *self
	}
}

impl<'a> HashDB<KeccakHasher, DBValue> for KVDatabase {
	fn get(&self, key: & <KeccakHasher as Hasher>::Out, prefix: Prefix) -> Option<DBValue> {
    // println!("memorydb get: {:?} {:?}", hex::encode(key), hex::encode(self.hashed_null_node.clone()));
		if key == &self.hashed_null_node {
      // println!("memorydb get d: {:?}", hex::encode(self.hashed_null_node.clone()));
			return Some(self.null_node_data.clone())
		}
		let key = prefixed_key::<KeccakHasher>(key, prefix);
		
		self.db.get(0, &key).expect("Database backend error")
	}

	fn contains(&self, hash: &<KeccakHasher as Hasher>::Out, prefix: Prefix) -> bool {
		// let hash = prefixed_key::<KeccakHasher>(hash, prefix);
		// if let Some(value) = self.overlay.get(&hash) {
		// 	return value.clone().is_some()
		// }
		// self.db.get(0, &hash).expect("Database backend error").is_some()
    // if hash == &self.hashed_null_node {
		// 	return true
		// }
    // self.get(&hash, prefix).is_some()
    if hash == &self.hashed_null_node {
      // println!("memorydb get d: {:?}", hex::encode(self.hashed_null_node.clone()));
			return true
		}
		let key = prefixed_key::<KeccakHasher>(hash, prefix);
		
		self.db.get(0, &key).expect("Database backend error").is_some()
	}

	fn insert(&mut self, prefix: Prefix, value: &[u8]) -> <KeccakHasher as Hasher>::Out {
		unimplemented!()
	}

	fn emplace(&mut self, key: <KeccakHasher as Hasher>::Out, prefix: Prefix, value: DBValue) {
    unimplemented!()
	}

	fn remove(&mut self, key: &<KeccakHasher as Hasher>::Out, prefix: Prefix) {
    unimplemented!()
	}
}

impl<'a> HashDBRef<KeccakHasher, DBValue> for KVDatabase
{
	fn get(&self, key: &<KeccakHasher as Hasher>::Out, prefix: Prefix) -> Option<DBValue> {
		HashDB::get(self, key, prefix)
	}
	fn contains(&self, key: &<KeccakHasher as Hasher>::Out, prefix: Prefix) -> bool {
		HashDB::contains(self, key, prefix)
	}
}

/// Derive a database key from hash value of the node (key) and  the node prefix.
pub fn prefixed_key<H: Hasher>(key: &H::Out, prefix: Prefix) -> Vec<u8> {
	let mut prefixed_key = Vec::with_capacity(key.as_ref().len() + prefix.0.len() + 1);
	prefixed_key.extend_from_slice(prefix.0);
	if let Some(last) = prefix.1 {
		prefixed_key.push(last);
	}
	prefixed_key.extend_from_slice(key.as_ref());
	prefixed_key
}