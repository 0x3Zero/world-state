use hash256_std_hasher::Hash256StdHasher;
use tiny_keccak::{Hasher, Keccak};

/// The `Keccak` hash output type.
pub type KeccakHash = [u8; 32];

/// Concrete implementation of Hasher using Keccak 256-bit hashes
#[derive(Debug)]
pub struct KeccakHasher;

impl hash_db::Hasher for KeccakHasher {
  type Out = KeccakHash;
  type StdHasher = Hash256StdHasher;
  const LENGTH: usize = 32;

  fn hash(x: &[u8]) -> Self::Out {
    keccak_256(x).into()
  }
}

/// Performs a Keccak-256 hash on the given input.
pub fn keccak_256(input: &[u8]) -> KeccakHash {
  let mut out = [0u8; 32];
  let mut k = Keccak::v256();
  k.update(input);
  k.finalize(&mut out);
  out
}