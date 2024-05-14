use super::{encode_amount, encode_bytes, ResponseValue};
use crate::evm::{Evm, SCALING_FACTOR};
use axum::response::Result;
use num_bigint::BigUint;
use serde_json::{json, Value};

pub async fn get_transactions(evm: Evm, address: [u8; 20]) -> Result<ResponseValue> {
    let transactions = evm.get_transactions_by_address(address).await?;

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
                    "value": encode_amount(BigUint::from(u128::from_be_bytes(signed_transaction.value().to_be_bytes())* SCALING_FACTOR as u128)),
                })}
            )
            .collect::<Vec<Value>>(),
    )))
}
