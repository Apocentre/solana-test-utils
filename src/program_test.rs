use solana_sdk::{
  system_program,
  rent::{Rent},
  system_instruction,
  program_error::ProgramError,
  clock::{Clock},
  program_pack::Pack,
  pubkey::Pubkey,
  signature::{Keypair, Signer},
  borsh::{try_from_slice_unchecked},
  transaction::Transaction,
  instruction::Instruction,
};
use spl_associated_token_account::{
  create_associated_token_account,
  get_associated_token_address,
};
use solana_program_test::{ProgramTestContext};
use crate::{
  time::{get_clock},
  tools::{clone_keypair, map_transaction_error}
};

pub struct ProgramTest {
  pub context: ProgramTestContext,
  pub rent: Rent,
  pub payer: Keypair,
  pub next_id: u8,
}

impl ProgramTest {
  pub async fn start_new(program_test: solana_program_test::ProgramTest) -> Self {
    let mut context = program_test.start_with_context().await;
    let rent = context.banks_client.get_rent().await.unwrap();
    let payer = clone_keypair(&context.payer);

    Self {
      context,
      rent,
      payer,
      next_id: 0,
    }
  }

  pub async fn process_transaction(
    &mut self,
    instructions: &[Instruction],
    signers: Option<&[&Keypair]>,
  ) -> Result<(), ProgramError> {
    let mut transaction = Transaction::new_with_payer(instructions, Some(&self.payer.pubkey()));
    let mut all_signers = vec![&self.payer];

    if let Some(signers) = signers {
      all_signers.extend_from_slice(signers);
    }

    let recent_blockhash = self
      .context
      .banks_client
      .get_latest_blockhash()
      .await
      .unwrap();

    transaction.sign(&all_signers, recent_blockhash);

    self.context
      .banks_client
      .process_transaction(transaction)
      .await
      .map_err(|e| map_transaction_error(e.into()))?;

    Ok(())
  }

  pub async fn create_account(&mut self) -> Keypair {
    let account = Keypair::new();
    let create_ix = system_instruction::create_account(
      &self.payer.pubkey(),
      &account.pubkey(),
      100_000_000_000_000,
      0,
      &system_program::ID,
    );

    self.process_transaction(&[create_ix], Some(&[&account]))
      .await
      .unwrap();

    account
  }

  pub async fn create_mint(
    &mut self,
    mint_keypair: &Keypair,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
  ) {
    let mint_rent = self.rent.minimum_balance(spl_token::state::Mint::LEN);

    let instructions = [
      system_instruction::create_account(
        &self.context.payer.pubkey(),
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

    self.process_transaction(&instructions, Some(&[mint_keypair]))
      .await
      .unwrap();
  }

  pub async fn create_associated_account(
    &mut self,
    wallet_address: &Pubkey,
    spl_token_mint_address: &Pubkey,
  ) {
    let ix = create_associated_token_account(
      &self.context.payer.pubkey(),
      wallet_address,
      spl_token_mint_address
    );

    self.process_transaction(&[ix], None)
      .await
      .unwrap();
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
    recipient_ata: &Pubkey,
  ) {
    // 1. create a new Mint account with 0 decimals
    self.create_mint(
      mint_keypair,
      &mint_authority.pubkey(),
      None,
      0
    ).await;

    // 2. mint 1 token into the recipient associated token account
    self.mint_tokens(
      &mint_keypair.pubkey(),
      &mint_authority,
      recipient_ata,
      1
    ).await;

    // 3. disable future minting by setting the mint authority to none
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

    self.process_transaction(&instructions, Some(&[mint_authority]))
      .await
      .unwrap();
  }

  pub async fn transfer_sol(&mut self, to_account: &Pubkey, lamports: u64) {
    let transfer_ix = system_instruction::transfer(
      &self.payer.pubkey(),
      to_account,
      lamports
    );

    self.process_transaction(&[transfer_ix], None)
      .await
      .unwrap();
  }

  pub async fn mint_tokens(
    &mut self,
    token_mint: &Pubkey,
    token_mint_authority: &Keypair,
    token_account: &Pubkey,
    amount: u64,
  ) {
    let mint_instruction = spl_token::instruction::mint_to(
      &spl_token::id(),
      token_mint,
      token_account,
      &token_mint_authority.pubkey(),
      &[],
      amount,
    )
    .unwrap();

    self.process_transaction(&[mint_instruction], Some(&[token_mint_authority]))
      .await
      .unwrap();
  }

  pub async fn get_account<T>(&mut self, account: Pubkey) -> T
  where T: borsh::de::BorshDeserialize
  {
    let mut account = self.context.banks_client.get_account(account).await.unwrap().unwrap();
    // Note! the first 8 bytes represent the Anchor account discriminator so we need to get rid of it first
    account.data.drain(0..8);
    
    try_from_slice_unchecked::<T>(&account.data).unwrap()
  }

  pub async fn get_clock(&mut self) -> Clock {
    get_clock(&mut self.context).await
  }
}
