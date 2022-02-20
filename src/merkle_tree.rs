use solana_sdk::keccak::hashv;
use std::convert::TryInto;
use rs_merkle::{
  Hasher,
  MerkleTree as MerkleTreeLib,
};

#[derive(Clone)]
pub struct SolanaHasher;

impl Hasher for SolanaHasher {
  type Hash = [u8; 32];

  fn hash(data: &[u8]) -> Self::Hash {
    println!("data ----> {:?}", data);

    // let data = data.to_vec();
    let left: [u8; 32] = data[..32].try_into().unwrap();
    let right: [u8; 32] = data[32..64].try_into().unwrap();

    println!("left----> {:?}", data);
    println!("right ----> {:?}", data);

    let mut sorted = vec![];

    if left <= right {
      sorted.append(&mut left.to_vec());
      sorted.append(&mut right.to_vec());
    } else {
      sorted.append(&mut right.to_vec());
      sorted.append(&mut left.to_vec());
    }

    println!("sorted ----> {:?}", sorted);

    hashv(&[data]).0
  }
}

pub struct MerkleTree {
  tree: MerkleTreeLib<SolanaHasher>
}

impl MerkleTree {
  pub fn new(mut leaves: Vec<[u8; 32]>) -> Self {
    // sort the leaves first
    leaves.sort();
    
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

  pub fn proof(&self, indices_to_prove: &[usize]) -> Vec<[u8; 32]> {
    self.tree.proof(&indices_to_prove).proof_hashes().to_vec()
  }

  /// Note this is the exact same logic that will be used in the on-chain program as well
  /// this is why we do not use the verify function from the underlying lib (rs_merkle)
  pub fn verify(proof: Vec<[u8; 32]>, root: [u8; 32], leaf: [u8; 32]) -> bool {
    let mut computed_hash = leaf;

    for proof_element in proof.into_iter() {
      if computed_hash <= proof_element {
        // Hash(current computed hash + current element of the proof)
        computed_hash = hashv(&[&computed_hash, &proof_element]).0;
      } else {
        // Hash(current element of the proof + current computed hash)
        computed_hash = hashv(&[&proof_element, &computed_hash]).0;
      }
    }
    // Check if the computed hash (root) is equal to the provided root
    computed_hash == root
  }
}
