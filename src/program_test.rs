use solana_program_runtime::invoke_context::ProcessInstructionWithContext;
use solana_sdk::{
  system_program,
  rent::{Rent},
  system_instruction,
  program_error::ProgramError,
  clock::{Clock, UnixTimestamp},
  pubkey::Pubkey,
  signature::{Keypair, Signer},
  borsh::{try_from_slice_unchecked},
  transaction::Transaction,
  instruction::Instruction,
};
use solana_program_test::{ProgramTestContext};
use crate::{
  time::{get_clock, advance_clock_past_timestamp},
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

  pub fn add_program(
    program_test: &mut solana_program_test::ProgramTest,
    program_name: &str,
    program_id: Pubkey,
    process_instruction: Option<ProcessInstructionWithContext>
  ) {
    program_test.add_program(program_name, program_id, process_instruction);
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

  pub async fn create_account(&mut self, space: u64) -> Keypair {
    let account = Keypair::new();
    let create_ix = system_instruction::create_account(
      &self.payer.pubkey(),
      &account.pubkey(),
      100_000_000_000_000,
      space,
      &system_program::ID,
    );

    self.process_transaction(&[create_ix], Some(&[&account]))
      .await
      .unwrap();

    account
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

  pub async fn advance_clock_past_timestamp(&mut self, unix_timestamp: UnixTimestamp) {
    advance_clock_past_timestamp(&mut self.context, unix_timestamp).await
  }
}
