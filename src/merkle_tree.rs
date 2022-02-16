use solana_sdk::keccak::hashv;
use rs_merkle::{Hasher, MerkleTree as MerkleTreeLib};

#[derive(Clone)]
pub struct SolanaHasher;

impl Hasher for SolanaHasher {
  type Hash = [u8; 32];

  fn hash(data: &[u8]) -> Self::Hash {
    hashv(&[data]).0
  }
}

pub struct MerkleTree {
  tree: MerkleTreeLib<SolanaHasher>
}

impl MerkleTree {
  pub fn new(leaves: Vec<[u8; 32]>) -> Self {
    Self {
      tree: MerkleTreeLib::<SolanaHasher>::from_leaves(&leaves)
    }
  }

  pub fn root(&self) -> Option<[u8; 32]> {
    self.tree.root()
  }

  pub fn root_hex(&self) -> Option<String> {
    self.tree.root_hex()
  }
}
