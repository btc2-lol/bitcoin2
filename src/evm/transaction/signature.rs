use borsh::{BorshDeserialize, BorshSerialize};
use k256::{ecdsa, ecdsa::RecoveryId};
use std::{
    io,
    io::{Read, Write},
};
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Signature(
    #[borsh(
        serialize_with = "encode_signature",
        deserialize_with = "decode_signature"
    )]
    pub ecdsa::Signature,
    #[borsh(
        serialize_with = "encode_recovery_id",
        deserialize_with = "decode_recovery_id"
    )]
    pub RecoveryId,
);

fn encode_signature<W: Write>(signature: &ecdsa::Signature, writer: &mut W) -> io::Result<()> {
    Ok(signature.to_bytes().serialize(writer)?)
}

fn encode_recovery_id<W: Write>(recovery_id: &RecoveryId, writer: &mut W) -> io::Result<()> {
    Ok(recovery_id.to_byte().serialize(writer)?)
}

fn decode_signature<R: Read>(reader: &mut R) -> Result<ecdsa::Signature, std::io::Error> {
    let bytes: Vec<u8> = BorshDeserialize::deserialize_reader(reader)?;
    ecdsa::Signature::from_slice(&bytes)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, ""))
}

fn decode_recovery_id<R: Read>(reader: &mut R) -> Result<RecoveryId, std::io::Error> {
    let byte = BorshDeserialize::deserialize_reader(reader)?;
    RecoveryId::from_byte(byte).ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, ""))
}