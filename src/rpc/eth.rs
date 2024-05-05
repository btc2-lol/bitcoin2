use super::{RequestValue, ResponseValue};
use crate::{
    constants::{CHAIN_ID, DEFAULT_GAS_LIMIT},
    error::Error,
    evm,
    evm::{scale_up, Evm},
};
use axum::response::Result;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use reth_primitives::U256;
use serde_json::{json, Value};

pub async fn chain_id() -> Result<ResponseValue> {
    Ok(ResponseValue::Number(U256::from::<i64>(CHAIN_ID)))
}
pub async fn send_raw_transaction(evm: Evm, raw_transaction: Vec<u8>) -> Result<ResponseValue> {
    let transaction =
        evm::TransactionSigned::decode_rlp_legacy_transaction(&mut &raw_transaction[..])
            .map_err(Error::from)?;

    evm.run_transaction(transaction).await?;

    Ok(ResponseValue::Null)
}

pub fn parse_i64(vec: &Vec<u8>) -> i64 {
    BigUint::from_bytes_be(vec).to_i64().unwrap()
}
pub fn encode_amount(amount: BigUint) -> Value {
    if amount == Zero::zero() {
        json!("0x0")
    } else {
        json!(format!(
            "0x{}",
            hex::encode(amount.to_bytes_be()).trim_start_matches('0')
        ))
    }
}

pub fn encode_bytes(bytes: &[u8]) -> Value {
    json!(format!("0x{}", hex::encode(bytes)))
}

pub async fn get_transaction_by_hash(transaction_hash: [u8; 32]) -> Result<ResponseValue> {
    Ok(ResponseValue::Value(json!({
      "blockHash":"0x0000000000000000000000000000000000000000000000000000000000000001",
      "blockNumber":"0x1",
      "cumulativeGasUsed": "0x0",
      "transactionIndex": "0x0",
      "effectiveGasPrice": "0x0",
      "transactionHash": encode_bytes(&transaction_hash),
      "status":"0x1",
      "logs": [],
      "gasUsed":"0x0",
    })))
}

pub async fn get_transaction_receipt(block_hash: [u8; 32]) -> Result<ResponseValue> {
    Ok(ResponseValue::Value(json!({
      "blockHash":"0x0000000000000000000000000000000000000000000000000000000000000001",
      "blockNumber":"0x1",
      "cumulativeGasUsed": "0x0",
      "transactionIndex": "0x0",
      "effectiveGasPrice": "0x0",
      "transactionHash": block_hash,
      "status":"0x1",
      "logs": [],
      "gasUsed":"0x0",
    })))
}
pub async fn get_transaction_count(evm: Evm, address: [u8; 20]) -> Result<ResponseValue> {
    Ok(ResponseValue::Number(U256::from::<i64>(
        evm.get_transaction_count_by_address(address).await,
    )))
}
pub async fn get_code(_block_hash: [u8; 32]) -> Result<ResponseValue> {
    Ok(ResponseValue::Value(json!("0x")))
}
pub async fn get_block_by_number(evm: Evm, block_number: i64) -> Result<ResponseValue> {
    if let Some(hash) = evm.get_transaction_hash(block_number).await {
        get_block_by_hash(evm, hash).await
    } else {
        get_block_by_hash(evm, [0; 32]).await
    }
}
pub async fn get_block_by_hash(_evm: Evm, block_hash: [u8; 32]) -> Result<ResponseValue> {
    Ok(ResponseValue::Value(json!({
       "hash": encode_bytes(&block_hash),
        "parentHash":encode_bytes(&[0; 32].to_vec()),
        "number": encode_amount(0u32.into()),
        "miner": encode_bytes(&[0; 32].to_vec()),
        "extraData": encode_bytes(&vec![]),
        "gasLimit": encode_amount(0u32.into()),
        "gasUsed": encode_amount(0u32.into()),
        "timestamp": encode_amount(0u32.into()),
        "transactions": vec![encode_bytes(&block_hash)],
    })))
}
pub async fn gas_price() -> Result<ResponseValue> {
    Ok(ResponseValue::Number(U256::from::<u64>(0)))
}

pub async fn estimate_gas() -> Result<ResponseValue> {
    Ok(ResponseValue::Number(U256::from::<i64>(DEFAULT_GAS_LIMIT)))
}

pub async fn call(_data: &Vec<u8>) -> Result<ResponseValue> {
    Ok(ResponseValue::Null)
}

pub async fn block_number(_evm: Evm, _block_number: &RequestValue) -> Result<ResponseValue> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let start = SystemTime::now();
    let timestamp = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    Ok(ResponseValue::Number(U256::from::<u64>(timestamp)))
}

pub async fn get_balance(evm: Evm, address: [u8; 20]) -> Result<ResponseValue> {
    let balance = evm.get_balance(address).await.unwrap_or(0);
    Ok(ResponseValue::Number(scale_up(balance)))
}
