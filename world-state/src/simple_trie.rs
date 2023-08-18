use std::{collections::HashMap, sync::Arc};

use hash_db::{AsHashDB, HashDB, Hasher, Prefix, HashDBRef};
use keccak_hasher::KeccakHasher;
use kvdb::{KeyValueDB, DBValue};

use crate::db::{KVDatabase, KVDB};

/// Immutable generated trie database with root.
pub struct SimpleTrie<'a> {
	// pub db: Arc<dyn KeyValueDB>,
  pub db: KVDB,
	pub overlay: &'a mut HashMap<Vec<u8>, Option<Vec<u8>>>,
  hashed_null_node: <KeccakHasher as Hasher>::Out,
	null_node_data: Vec<u8>,
}

impl<'a> SimpleTrie<'a> {
	/// Create a new instance of `Self`.
	pub fn new(db: KVDB, overlay: &'a mut HashMap<Vec<u8>, Option<Vec<u8>>>) -> Self {

    SimpleTrie {
      db,
      overlay,
      hashed_null_node: KeccakHasher::hash(&[0u8]),
      null_node_data: [0u8][..].into()
    }
	}
}

impl<'a> AsHashDB<KeccakHasher, DBValue> for SimpleTrie<'a> {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> {
		self
	}

	fn as_hash_db_mut<'b>(&'b mut self) -> &'b mut (dyn HashDB<KeccakHasher, DBValue> + 'b) {
		&mut *self
	}
}

impl<'a> HashDB<KeccakHasher, DBValue> for SimpleTrie<'a> {
	fn get(&self, key: & <KeccakHasher as Hasher>::Out, prefix: Prefix) -> Option<DBValue> {
    // println!("memorydb get: {:?} {:?}", hex::encode(key), hex::encode(self.hashed_null_node.clone()));
		if key == &self.hashed_null_node {
      // println!("memorydb get d: {:?}", hex::encode(self.hashed_null_node.clone()));
			return Some(self.null_node_data.clone())
		}
		let key = prefixed_key::<KeccakHasher>(key, prefix);
		if let Some(value) = self.overlay.get(&key) {
			return value.clone()
		}
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
		if let Some(value) = self.overlay.get(&key) {
			return value.clone().is_some()
		}
		self.db.get(0, &key).expect("Database backend error").is_some()
	}

	fn insert(&mut self, prefix: Prefix, value: &[u8]) -> <KeccakHasher as Hasher>::Out {
		let key = KeccakHasher::hash(value);
		self.emplace(key, prefix, value.to_vec());
		key
	}

	fn emplace(&mut self, key: <KeccakHasher as Hasher>::Out, prefix: Prefix, value: DBValue) {
    if value == self.null_node_data {
			return
		}
		let key = prefixed_key::<KeccakHasher>(&key, prefix);
    // println!("key: {:?}", key);
		self.overlay.insert(key, Some(value));
	}

	fn remove(&mut self, key: &<KeccakHasher as Hasher>::Out, prefix: Prefix) {
    if key == &self.hashed_null_node {
			return
		}
		let key = prefixed_key::<KeccakHasher>(key, prefix);
		self.overlay.insert(key, None);
	}
}

impl<'a> HashDBRef<KeccakHasher, DBValue> for SimpleTrie<'a>
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