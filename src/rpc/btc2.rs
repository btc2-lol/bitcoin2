use super::{encode_bytes, ResponseValue};
use crate::db;
use axum::response::Result;

use crate::{evm::scale_up, rpc::encode_u256};
use serde_json::{json, Value};

use sqlx::PgPool;

pub async fn get_transactions(pool: PgPool, address: [u8; 20]) -> Result<ResponseValue> {
    let entries = db::get_ledger_by_address(&pool, address).await?;
    Ok(ResponseValue::Value(serde_json::Value::Array(
        entries
            .into_iter()
            .map(|entry| {
                json!(
                {
                    "creditor": encode_bytes(&entry.creditor),
                    "debtor": encode_bytes(&entry.debtor),
                    "value": encode_u256(scale_up(entry.value)),
                })
            })
            .collect::<Vec<Value>>(),
    )))
}
