use anchor_spl::token::{TokenAccount};
use anchor_lang::{
  AccountDeserialize,
};
use solana_program_test::{ProgramTestContext};
use solana_sdk::{
  pubkey::Pubkey,
};

pub struct Spl<'a> {
  pub context: &'a mut ProgramTestContext,
}

impl<'a> Spl<'a> {
  pub fn new(context: &'a mut ProgramTestContext) -> Self {
    Self {
      context
    }
  }

  pub async fn get_token_account(&mut self, token_account: Pubkey) -> TokenAccount {
    let account = self.context.banks_client.get_account(token_account).await.unwrap().unwrap();
    TokenAccount::try_deserialize_unchecked(&mut account.data.as_ref()).unwrap()
  }
}
