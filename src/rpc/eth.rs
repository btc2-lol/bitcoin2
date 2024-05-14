use super::ResponseValue;
use crate::{
    constants::{CHAIN_ID, DEFAULT_GAS_LIMIT},
    error::Error,
    evm,
    evm::{scale_up, Evm},
};
use axum::response::Result;
use num_bigint::BigUint;
use num_traits::Zero;
use reth_primitives::U256;
use serde_json::{json, Value};

pub async fn chain_id() -> Result<ResponseValue> {
    Ok(ResponseValue::Number(U256::from::<i64>(CHAIN_ID)))
}
pub async fn send_raw_transaction(evm: Evm, raw_transaction: Vec<u8>) -> Result<ResponseValue> {
    let transaction =
        evm::TransactionSigned::decode_rlp_legacy_transaction(&mut &raw_transaction[..])
            .map_err(Error::from)?;

    let transaction_id = evm.run_transaction(transaction).await?;

    Ok(ResponseValue::Value(id_as_hash(transaction_id)))
}

pub fn id_as_hash(id: i64) -> Value {
    let mut array = [0u8; 32];
    array[24..].copy_from_slice(&id.to_be_bytes());

    encode_bytes(&array)
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
pub async fn get_block_by_number(_evm: Evm, block_number: i64) -> Result<ResponseValue> {
    println!("getting block by number");
    Ok(ResponseValue::Value(json!({
       "hash": id_as_hash(block_number),
        "parentHash":encode_bytes(&[0; 32].to_vec()),
        "number": encode_amount(0u32.into()),
        "miner": encode_bytes(&[0; 32].to_vec()),
        "extraData": encode_bytes(&vec![]),
        "gasLimit": encode_amount(0u32.into()),
        "gasUsed": encode_amount(0u32.into()),
        "timestamp": encode_amount(0u32.into()),
        "transactions": vec![id_as_hash(block_number)],
    })))
}
pub async fn get_block_by_hash(evm: Evm, block_hash: [u8; 32]) -> Result<ResponseValue> {
    get_block_by_number(
        evm,
        i64::from_be_bytes(block_hash[28..].try_into().unwrap()),
    )
    .await
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

pub async fn block_number(evm: Evm) -> Result<ResponseValue> {
    let transaction_count = evm.get_transaction_count().await.unwrap_or(0);
    // use std::time::{SystemTime, UNIX_EPOCH};

    // let start = SystemTime::now();
    // let timestamp = start
    //     .duration_since(UNIX_EPOCH)
    //     .expect("Time went backwards")
    //     .as_secs();

    Ok(ResponseValue::Number(U256::from::<u64>(
        (transaction_count).try_into().unwrap(),
    )))
}

pub async fn get_balance(evm: Evm, address: [u8; 20]) -> Result<ResponseValue> {
    let balance = evm.get_balance(address).await.unwrap_or(0);
    Ok(ResponseValue::Number(scale_up(balance)))
}
