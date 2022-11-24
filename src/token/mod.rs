use std::error::Error;

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HS256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct Token {
    pub id: [u8; 16],
    pub timestamp: i64,
}

pub fn verify(input: &str, secret: &str) -> Result<Token, Box<dyn Error>> {
    if input.len() < 32 {
        return Err("token too short".into());
    }

    let mut payload: Vec<u8> = vec![];

    base64::decode_config_buf(&input[..32], base64::URL_SAFE_NO_PAD, &mut payload)?;

    let mut mac = HS256::new_from_slice(secret.as_bytes())?;

    mac.update(&payload);

    let signature = mac.finalize().into_bytes();

    if &input[32..] != base64::encode_config(signature, base64::URL_SAFE_NO_PAD) {
        return Err("".into());
    }

    let mut id = [0 as u8; 16];

    id.swap_with_slice(&mut payload[..16]);

    let mut timestamp = [0 as u8; 8];

    timestamp.swap_with_slice(&mut payload[16..24]);

    let timestamp = i64::from_le_bytes(timestamp);

    Ok(Token {
        id,
        timestamp
    })
}