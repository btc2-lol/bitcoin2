use super::decode_varint;

use axum::http::StatusCode;
use digest::Digest;
use k256::ecdsa::VerifyingKey;
use ripemd::Ripemd160;
// use rusqlite::{params, Connection};
// use crate::http::Error;
use sha2::Sha256;
use std::io::Cursor;

#[derive(Debug)]
pub struct Vout {
    pub transaction_id: [u8; 32],
    pub index: u16,
}

pub fn validate(_vout: &Vout, _verifying_key: VerifyingKey) -> Result<u64, crate::http::Error> {
    Ok(0)
    // let conn = Connection::open("utxos.sqlite").unwrap();
    // let sql = "SELECT amount, compressed_script from utxos WHERE transaction_id = ?";
    // let (amount, compressed_script): (u64, Vec<u8>) = conn
    //     .query_row(sql, params![vout.transaction_id], |row| {
    //         Ok((row.get(0)?, row.get(1)?))
    //     })
    //     .map_err(|_| StatusCode::BAD_REQUEST)?;
    // validate_compressed_script(compressed_script, verifying_key)?;
    // Ok(amount)
}

pub fn validate_compressed_script(
    compressed_script: Vec<u8>,
    verifying_key: VerifyingKey,
) -> Result<(), StatusCode> {
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(compressed_script.clone().into());
    let nsize = decode_varint(&mut cursor).map_err(|_| StatusCode::UNAUTHORIZED)?;
    match nsize {
        28 => validate_p2wpkh(compressed_script[1..].to_vec(), verifying_key),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

pub fn validate_p2wpkh(
    compressed_script: Vec<u8>,
    verifying_key: VerifyingKey,
) -> Result<(), StatusCode> {
    let hashed_verifying_key = Ripemd160::digest(Sha256::digest(verifying_key.to_sec1_bytes()));

    if compressed_script[2..] == hashed_verifying_key.to_vec() {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
