use super::decode_varint;

use digest::Digest;

use crate::error::{Error, Result};
use ripemd::Ripemd160;
use rusqlite::{params, Connection};
use sha2::Sha256;
use std::io::Cursor;

#[derive(Debug)]
pub struct Outpoint {
    pub hash: [u8; 32],
    pub index: u16,
}

pub fn validate(vout: &Outpoint, unlocking_script: &[u8]) -> Result<i64> {
    let conn = Connection::open("utxos.sqlite").unwrap();
    let sql = "SELECT amount, compressed_script from utxos WHERE transaction_id = ? AND vout = ?";
    let (amount, compressed_script): (Result<i64>, Result<Vec<u8>>) = conn
        .query_row(sql, params![vout.hash, vout.index], |row| {
            Ok((
                row.get(0).map_err(|e| Error::Error(e.to_string())),
                row.get(1).map_err(|e| Error::Error(e.to_string())),
            ))
        })
        .map_err(|e| Error::Error(e.to_string()))?;
    validate_compressed_script(compressed_script?, unlocking_script)?;
    Ok(amount?)
}

pub fn validate_compressed_script(
    compressed_script: Vec<u8>,
    unlocking_script: &[u8],
) -> Result<()> {
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(compressed_script.clone().into());
    let nsize = decode_varint(&mut cursor).map_err(|e| Error::Error(e.to_string()))?;
    match nsize {
        28 => validate_p2wpkh(compressed_script[1..].to_vec(), unlocking_script),
        _ => Err(Error::InvalidScript),
    }
}

pub fn validate_p2wpkh(compressed_script: Vec<u8>, unlocking_script: &[u8]) -> Result<()> {
    let hashed_verifying_key = Ripemd160::digest(Sha256::digest(unlocking_script[65..].to_vec()));

    if compressed_script[2..] == hashed_verifying_key[..] {
        Ok(())
    } else {
        Err(Error::InvalidScript)
    }
}
