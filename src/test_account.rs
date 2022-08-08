use futures::{stream, StreamExt};
use std::sync::{Arc};
use solana_program_test::{tokio::sync::{Mutex}};
use solana_sdk::{
  signature::{Keypair, Signer},
  native_token::sol_to_lamports,
};
use crate::{
  program_test::ProgramTest,
};

#[derive(Default)]
pub struct TestAccount {
  pub participants: Vec<Keypair>,
}

impl TestAccount {
  pub async fn new(pt: &mut ProgramTest, count: i32) -> Self {
    let participants: Vec<Keypair> = (0..count)
      .map(|_| Keypair::new())
      .collect();
      
    // fund the newly created accounts
    let pt = Arc::new(Mutex::new(pt));
    stream::iter(participants.iter())
      .for_each(|account| {
        let pt = Arc::clone(&pt);

        async move {
          let mut lock = pt.lock().await;
          lock.airdrop(&account.pubkey(), sol_to_lamports(100_f64)).await
        }
      })
      .await;

    Self {
      participants,
    }
  }
}
