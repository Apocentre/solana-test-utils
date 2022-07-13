use std::{mem::size_of};
use bytemuck::{
  from_bytes,
  Pod,
};

pub async fn deser_zero_account<'a, T: Pod>(data: &'a [u8],) -> &'a T {
  let data = &data[8..size_of::<T>() + 8];
  
  from_bytes::<T>(&data)
}
