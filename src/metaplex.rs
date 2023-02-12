use std::sync::Arc;
use solana_program_test::{tokio::sync::{Mutex}};
use solana_sdk::{
  pubkey::Pubkey,
  signature::{Keypair, Signer},
};
use mpl_token_metadata::{
  state::{Uses, Creator, Collection},
  instruction::{
    create_metadata_accounts_v3,
  },
};
use crate::{
  program_test::ProgramTest,
};

pub struct CreateMetadataAccounts<'a> {
  pub mint: Pubkey,
  pub mint_authority: &'a Keypair,
  pub metadata_account: Pubkey,
  pub payer: &'a Keypair,
  pub update_authority: &'a Keypair,
}

pub struct Metaplex {
  pub program_test: Arc<Mutex<ProgramTest>>
}


impl Metaplex {
  pub fn new(program_test: Arc<Mutex<ProgramTest>>) -> Self {
    Self {
      program_test
    }
  }

  pub async fn create_metadata<'a>(
    &mut self,
    accounts: CreateMetadataAccounts<'a>,
    name: String, 
    symbol: String,
    uri: String,
    creators: Option<Vec<Creator>>, 
    seller_fee_basis_points: u16, 
    update_authority_is_signer: bool, 
    is_mutable: bool, 
    collection: Option<Collection>, 
    uses: Option<Uses>
  ) {
    let ix = create_metadata_accounts_v3(
      mpl_token_metadata::ID,
      accounts.metadata_account,
      accounts.mint,
      accounts.mint_authority.pubkey(),
      accounts.payer.pubkey(),
      accounts.update_authority.pubkey(),
      name,
      symbol,
      uri,
      creators,
      seller_fee_basis_points,
      update_authority_is_signer,
      is_mutable,
      collection,
      uses,
      None,
    );
  
    let mut lock_pt = self.program_test.lock().await;
    let signers = &[
      accounts.mint_authority,
      accounts.payer,
      accounts.update_authority
    ];
    lock_pt.process_transaction(&[ix], Some(signers)).await.unwrap();
  }
}
