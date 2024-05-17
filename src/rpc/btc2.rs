use super::{encode_amount, encode_bytes, ResponseValue};
use crate::{db, evm::SCALING_FACTOR};
use axum::response::Result;
use num_bigint::BigUint;
use serde_json::{json, Value};
use num_traits::FromPrimitive;
use sqlx::PgPool;

pub async fn get_transactions(pool: PgPool, address: [u8; 20]) -> Result<ResponseValue> {
    let transactions = db::get_transactions_by_address(&pool, address).await?;
    Ok(ResponseValue::Value(serde_json::Value::Array(
        transactions
            .into_iter()
            .map(|signed_transaction|{
                let to = if let Some(to) = signed_transaction.to() {
                    encode_bytes(&to.to_vec())
                }else {
                    Value::Null};let from = if let Some(from) = signed_transaction.recover_signer() {
                    encode_bytes(&from.to_vec())
                }else {
                    Value::Null
                };
                json!(
                {
                    "to": to,
                    "from": from,
                    "value": encode_amount(BigUint::from_bytes_be(&signed_transaction.value().to_be_bytes_vec())* BigUint::from_i64(SCALING_FACTOR).unwrap()),
                })}
            )
            .collect::<Vec<Value>>(),
    )))
}
