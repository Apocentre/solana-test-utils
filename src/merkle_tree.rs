use solana_sdk::keccak::hashv;
use std::hash::Hasher;
use merkletree::{
  merkle::MerkleTree as OrigMerkleTree,
  store::VecStore,
  hash::{Algorithm}
};

#[derive(Default)]
pub struct SolanaHasher {
  val: Vec<u8>
}

impl Hasher for SolanaHasher {
  #[inline]
  fn write(&mut self, msg: &[u8]) {
    self.val = msg.to_vec();
  }

  #[inline]
  fn finish(&self) -> u64 {
    unimplemented!()
  }
}

impl Algorithm<[u8; 32]> for SolanaHasher {
  #[inline]
  fn hash(&mut self) -> [u8; 32] {
    hashv(&[self.val.as_slice()]).0
  }

  #[inline]
  fn reset(&mut self) {
    self.val = vec![];
  }
}

pub struct MerkleTree {
  tree: OrigMerkleTree<[u8; 32], SolanaHasher, VecStore<[u8; 32]>>
}

impl MerkleTree {
  pub fn new(leaves: Vec<[u8; 32]>) -> Self {
    Self {
      tree: OrigMerkleTree::from_data(leaves).unwrap(),
    }
  }

  pub fn root(&self) -> [u8; 32] {
    self.tree.root()
  }
}
