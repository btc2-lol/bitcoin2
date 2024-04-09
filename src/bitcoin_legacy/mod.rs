mod sha256d;
pub mod utxos;

use crate::StatusCode;
use digest::Digest;
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use sha256d::Sha256d;
use std::io::{ErrorKind, Read};

// https://github.com/bitcoin/bitcoin/blob/c8e3978114716bb8fb10695b9d187652f3ab4926/src/pubkey.cpp#L287
// ¯\_(ツ)_/¯

pub fn decode_recovery_id_byte(recovery_byte: u8) -> u8 {
    (recovery_byte - 27) & 0x03
}
// https://github.com/bitcoin/bitcoin/blob/d1e9a02126634f9e2ca0b916b69b173a8646524d/src/util/message.cpp#L23
const MESSAGE_MAGIC: &'static str = "Bitcoin Signed Message:\n";

pub fn recover_from_msg(
    message: Vec<u8>,
    signature: [u8; 65],
) -> Result<VerifyingKey, crate::http::Error> {
    let recovery_byte = decode_recovery_id_byte(signature[0]);
    let recovery_id = RecoveryId::from_byte(recovery_byte)
        .ok_or(crate::http::Error(StatusCode::BAD_REQUEST, "".to_string()))?;
    let digest = signed_message_digest(message.clone().into());

    Ok(VerifyingKey::recover_from_digest(
        digest,
        &Signature::from_bytes(signature[1..].into())?,
        recovery_id,
    )?)
}

fn signed_message_digest(message: Vec<u8>) -> Sha256d {
    let hasher = Sha256d::new_with_prefix(
        [
            encode_varint(MESSAGE_MAGIC.len()),
            MESSAGE_MAGIC.into(),
            encode_varint(message.len()),
            message,
        ]
        .concat(),
    );

    return hasher;
}

// https://github.com/bitcoin/bitcoin/blob/c8e3978114716bb8fb10695b9d187652f3ab4926/src/leveldb/util/coding.cc#L21
fn encode_varint(value: usize) -> Vec<u8> {
    match value {
        0..=0xfc => vec![value as u8],
        0xfd..=0xffff => [&[0xfd], &(value as u16).to_le_bytes()[..]].concat(),
        0x10000..=0xffffffff => [&[0xfe], &(value as u32).to_le_bytes()[..]].concat(),
        _ => [&[0xff], &value.to_le_bytes()[..]].concat(),
    }
}
//https://github.com/bitcoin/bitcoin/blob/4cc99df44aec4d104590aee46cf18318e22a8568/src/serialize.h#L464-L484

fn decode_varint<R>(reader: &mut R) -> std::io::Result<u64>
where
    R: Read,
{
    let mut n = u64::from(0u8);
    loop {
        let mut buffer = [0; 1];
        reader.read_exact(&mut buffer)?;
        let ch_data = u64::from(buffer[0]);
        n = (n << 7) | (ch_data & u64::from(0x7Fu8));
        if ch_data & u64::from(0x80u8) != u64::from(0u8) {
            if n == u64::MAX {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "ReadVarInt: size too large",
                ));
            }
            n = n + 1;
        } else {
            return Ok(n);
        }
    }
}

#[cfg(test)]
mod tests {
    use digest::Digest;
    use hex_lit::hex;
    #[test]
    fn signed_message_digest() {
        assert_eq!(
            super::signed_message_digest("test".into())
                .finalize()
                .to_vec(),
            hex!("9ce428d58e8e4caf619dc6fc7b2c2c28f0561654d1f80f322c038ad5e67ff8a6")
        )
    }
}
