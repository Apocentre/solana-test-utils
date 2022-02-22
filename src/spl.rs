use anchor_spl::token::{TokenAccount};
use anchor_lang::{
  AccountDeserialize,
};
use spl_associated_token_account::{
  create_associated_token_account,
  get_associated_token_address,
};
use solana_sdk::{
  pubkey::Pubkey,
  signature::{Keypair, Signer},
  system_instruction,
  program_pack::Pack,
};
use crate::{
  program_test::ProgramTest,
};

pub struct Spl<'a> {
  pub program_test: &'a mut ProgramTest,
}

impl<'a> Spl<'a> {
  pub fn new(program_test: &'a mut ProgramTest) -> Self {
    Self {
      program_test
    }
  }

  pub async fn get_token_account(&mut self, token_account: Pubkey) -> TokenAccount {
  
    let account = self
      .program_test
      .context
      .banks_client
      .get_account(token_account)
      .await.unwrap().unwrap();

    TokenAccount::try_deserialize_unchecked(&mut account.data.as_ref()).unwrap()
  }

  pub async fn mint_tokens(
    &mut self,
    token_mint: &Pubkey,
    token_mint_authority: &Keypair,
    token_account: &Pubkey,
    amount: u64,
  ) {
    let ix = spl_token::instruction::mint_to(
      &spl_token::id(),
      token_mint,
      token_account,
      &token_mint_authority.pubkey(),
      &[],
      amount,
    )
    .unwrap();

    self.program_test.process_transaction(&[ix], Some(&[token_mint_authority]))
      .await
      .unwrap();
  }

  pub async fn transfer(
    &mut self,
    from: &Pubkey, 
    to: &Pubkey,
    authority: &Keypair,
    amount: u64,
  ) {
    let ix = spl_token::instruction::transfer(
      &spl_token::id(),
      from,
      to,
      &authority.pubkey(),
      &[&authority.pubkey()],
      amount,
    )
    .unwrap();

    self.program_test.process_transaction(&[ix], Some(&[authority]))
      .await
      .unwrap();
  }

  pub async fn create_mint(
    &mut self,
    mint_keypair: &Keypair,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
  ) {
    let mint_rent = self.program_test.rent.minimum_balance(spl_token::state::Mint::LEN);

    let instructions = [
      system_instruction::create_account(
        &self.program_test.context.payer.pubkey(),
        &mint_keypair.pubkey(),
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(),
      ),
      spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_keypair.pubkey(),
        mint_authority,
        freeze_authority,
        decimals,
      )
      .unwrap(),
    ];

    self.program_test.process_transaction(&instructions, Some(&[mint_keypair]))
      .await
      .unwrap();
  }

  pub async fn create_associated_account(
    &mut self,
    wallet_address: &Pubkey,
    spl_token_mint_address: &Pubkey,
  ) -> Pubkey {
    let ix = create_associated_token_account(
      &self.program_test.context.payer.pubkey(),
      wallet_address,
      spl_token_mint_address
    );

    self.program_test.process_transaction(&[ix], None)
      .await
      .unwrap();

    get_associated_token_address(wallet_address, spl_token_mint_address)
  }

  pub fn get_associated_token_address(
    wallet_address: &Pubkey, 
    spl_token_mint_address: &Pubkey
  ) -> Pubkey {
    get_associated_token_address(wallet_address, spl_token_mint_address)
  }

  pub async fn create_nft(
    &mut self,
    mint_keypair: &Keypair,
    mint_authority: &Keypair,
    recipient: &Pubkey,
  ) {
    // 1. create a new Mint account with 0 decimals
    self.create_mint(
      mint_keypair,
      &mint_authority.pubkey(),
      None,
      0
    ).await;

    // 2. create a new associated token account
    let mint_account = mint_keypair.pubkey();
    self.create_associated_account(recipient, &mint_account).await;

    // 3. mint 1 token into the recipient associated token account
    self.mint_tokens(
      &mint_keypair.pubkey(),
      &mint_authority,
      &Self::get_associated_token_address(recipient, &mint_account),
      1
    ).await;

    // 4. disable future minting by setting the mint authority to none
    self.set_mint_authority(&mint_keypair, &mint_authority).await;
  }

  pub async fn set_mint_authority(
    &mut self,
    mint_keypair: &Keypair,
    mint_authority: &Keypair,
  ) {
    let instructions = [
      spl_token::instruction::set_authority(
        &spl_token::id(),
        &mint_keypair.pubkey(),
        None,
        spl_token::instruction::AuthorityType::MintTokens,
        &mint_authority.pubkey(),
        &[&mint_authority.pubkey()]
      )
      .unwrap(),
    ];

    self.program_test.process_transaction(&instructions, Some(&[mint_authority]))
      .await
      .unwrap();
  }
}
