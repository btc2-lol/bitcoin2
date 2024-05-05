use super::{encode_bytes, ResponseValue};
use crate::evm::Evm;
use axum::response::Result;
use serde_json::{json, Value};

pub async fn get_transactions(evm: Evm, address: [u8; 20]) -> Result<ResponseValue> {
    let transactions = evm.get_transactions_by_address(address).await?;

    Ok(ResponseValue::Value(serde_json::Value::Array(
        transactions
            .into_iter()
            .map(|signed_transaction| json!(encode_bytes(&alloy_rlp::encode(signed_transaction))))
            .collect::<Vec<Value>>(),
    )))
}
