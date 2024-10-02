use soroban_sdk::{Bytes, BytesN, Env};

pub fn u128_to_bytes(env: &Env, value: u128) -> Bytes {
    let bytes_array: [u8; 16] = value.to_be_bytes();
    let bytes_n: BytesN<16> = BytesN::from_array(env, &bytes_array);
    bytes_n.into()
}