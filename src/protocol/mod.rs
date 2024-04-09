use http::Result;
pub mod legacy_transfer_by_message;
use crate::bitcoin_legacy;
use crate::http;
use axum::body::Bytes;
use axum::http::StatusCode;
use k256::ecdsa::Signature;
use k256::ecdsa::VerifyingKey;
pub use legacy_transfer_by_message::*;
use num_enum::TryFromPrimitive;

const SIGNATURE_LENGTH: usize = 65;

pub fn recover_verifying_key(message: &mut Bytes) -> Result<VerifyingKey> {
    let signature = split_off_signature(message)?;

    if MessageType::from_bytes(message)? == MessageType::LegacyTransferByMessage {
        bitcoin_legacy::recover_from_msg(message[1..].to_vec().clone().into(), signature)
    } else {
        recover_from_msg(message, signature)
    }
}

fn recover_from_msg(message: &mut Bytes, signature: [u8; 65]) -> Result<VerifyingKey> {
    Ok(VerifyingKey::recover_from_msg(
        message,
        &Signature::from_bytes(signature[1..].into()).unwrap(),
        signature[0].try_into()?,
    )?)
}

fn split_off_signature(message: &mut Bytes) -> Result<[u8; SIGNATURE_LENGTH]> {
    Ok(message
        .split_off(message.len() - SIGNATURE_LENGTH)
        .to_vec()
        .try_into()
        .map_err(|_| {
            http::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to split off signature".to_string(),
            )
        })?)
}
#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum MessageType {
    LegacyTransferByMessage,
}

impl MessageType {
    pub fn from_bytes(message: &Bytes) -> Result<Self> {
        Ok(message
            .get(0)
            .ok_or(http::err("Message is empty"))?
            .clone()
            .try_into()?)
    }
}

#[derive(Debug)]
pub enum Message {
    LegacyTransferByMessage(LegacyTransferByMessage),
}

impl Message {
    pub fn from_bytes(bytes: Bytes) -> Result<Self> {
        Ok(match MessageType::from_bytes(&bytes)? {
            MessageType::LegacyTransferByMessage => {
                Self::LegacyTransferByMessage(LegacyTransferByMessage::from_bytes(&bytes[1..])?)
            }
        })
    }

    pub fn execute(self, verifying_key: VerifyingKey) -> Result<()> {
        match self {
            Self::LegacyTransferByMessage(message) => message.execute(verifying_key),
        }
    }
}
