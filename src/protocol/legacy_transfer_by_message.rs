use crate::{
    bitcoin_legacy::utxos::{self, Vout},
    http,
};
use axum::http::StatusCode;
use http::Result;
use k256::ecdsa::VerifyingKey;
use sqlx::PgPool;
use std::{io, io::BufRead};

#[derive(Default, Debug)]
pub struct LegacyTransferByMessage {
    pub vouts: Vec<Vout>,
    pub to: [u8; 20],
    pub amount: i64,
}

impl LegacyTransferByMessage {
    pub fn _execute(&self, _pool: PgPool, verifying_key: VerifyingKey) -> Result<()> {
        let validated_amount = self
            .vouts
            .iter()
            .map(|utxo| utxos::validate(utxo, verifying_key))
            .sum::<Result<i64>>();
        if self.amount == validated_amount? {
            // db::transfer(&pool, LEGACY_ACCOUNT, self.to, self.amount);
            Ok(())
        } else {
            Err(http::err("Unauthorized amount"))
        }
    }
}

impl LegacyTransferByMessage {
    pub fn _from_bytes(bytes: &[u8]) -> Result<Self> {
        let cursor = io::Cursor::new(bytes);
        let buffered = io::BufReader::new(cursor);
        let mut lines = buffered.lines();
        let vout_count: usize = lines
            .next()
            .ok_or(http::err("no newlines in legacy signed message"))??
            .parse()?;

        let vouts = lines
            .by_ref()
            .take(vout_count)
            .map(|vout_bytes| {
                let (tx_id_bytes, index_bytes) = vout_bytes.as_ref()?.split_at(64);

                let mut transaction_id: [u8; 32] = hex::decode(String::from(tx_id_bytes))?
                    .try_into()
                    .map_err(|_| http::err("invalid transaction length"))?;
                transaction_id.reverse();
                Ok(Vout {
                    transaction_id,
                    index: index_bytes.parse()?,
                })
            })
            .collect::<Result<_>>()?;
        let to = hex::decode(lines.next().ok_or(http::Error(
            StatusCode::BAD_REQUEST,
            "no to address specified".to_string(),
        ))??)?
        .try_into()
        .map_err(|_| {
            http::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid to length".to_string(),
            )
        })?;
        let amount = lines
            .next()
            .ok_or(http::Error(
                StatusCode::BAD_REQUEST,
                "amounts underflow".to_string(),
            ))??
            .parse()?;

        Ok(Self { vouts, to, amount })
    }
}
