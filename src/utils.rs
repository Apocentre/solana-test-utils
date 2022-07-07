pub fn unit(decimals: u8) -> u64 {
  10 ^ decimals as u64
}

pub fn to_base(val: u64, decimals: u8) -> u64 {
  val * unit(decimals)
}
