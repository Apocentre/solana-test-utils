use std::borrow::Borrow;
use solana_sdk::{
  clock::{Clock, UnixTimestamp},
  sysvar,
};
use solana_program_test::{
  ProgramTestContext,
};
use bincode::deserialize;

#[allow(dead_code)]
pub async fn get_clock(context: &mut ProgramTestContext) -> Clock {
  let clock_account = sysvar::clock::id();

  context
    .banks_client
    .get_account(clock_account)
    .await
    .unwrap()
    .map(|a| deserialize::<Clock>(a.data.borrow()).unwrap())
    .unwrap_or_else(|| panic!("GET-TEST-ACCOUNT-ERROR: Account {}", clock_account))
}

#[allow(dead_code)]
pub async fn advance_clock_past_timestamp(context: &mut ProgramTestContext, unix_timestamp: UnixTimestamp) {
  let mut clock = get_clock(context).await;
  let mut n = 1;

  while clock.unix_timestamp <= unix_timestamp {
    // Since the exact time is not deterministic keep wrapping by arbitrary 400 slots until we pass the requested timestamp
    context
      .warp_to_slot(clock.slot + n * 400)
      .unwrap();

    n += 1;
    clock = get_clock(context).await;
  }
}

#[allow(dead_code)]
pub async fn advance_clock_by_min_timespan(context: &mut ProgramTestContext, time_span: u64) {
  let clock = get_clock(context).await;
  advance_clock_past_timestamp(context, clock.unix_timestamp + (time_span as i64)).await;
}

#[allow(dead_code)]
pub async fn advance_clock(context: &mut ProgramTestContext) {
  let clock = get_clock(context).await;
  context.warp_to_slot(clock.slot + 2).unwrap();
}
