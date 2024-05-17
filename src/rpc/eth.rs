use super::ResponseValue;
use crate::{
    constants::{CHAIN_ID, DEFAULT_GAS_LIMIT},
    db,
    db::TransactionSignedRow,
    error::Error,
    evm,
    evm::{scale_up, Evm},
};
use axum::response::Result;
use num_bigint::BigUint;
use num_traits::Zero;
use reth_primitives::U256;
use serde_json::{json, Value};
use sqlx::PgPool;

pub async fn chain_id() -> Result<ResponseValue> {
    Ok(ResponseValue::Number(U256::from::<i64>(CHAIN_ID)))
}
pub async fn send_raw_transaction(pool: PgPool, raw_transaction: Vec<u8>) -> Result<ResponseValue> {
    let transaction =
        evm::TransactionSigned::decode_rlp_legacy_transaction(&mut &raw_transaction[..])
            .map_err(Error::from)?;

    let evm: Evm = Evm::new(pool);
    evm.run_transaction(&transaction).await?;

    Ok(ResponseValue::Value(encode_bytes(
        &transaction.hash().to_vec(),
    )))
}

pub fn encode_u256(amount: U256) -> Value {
    if amount == U256::ZERO {
        json!("0x0")
    } else {
        json!(format!(
            "0x{}",
            hex::encode(amount.to_be_bytes_vec()).trim_start_matches('0')
        ))
    }
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
      "blockHash": encode_bytes(&transaction_hash),
      "hash": encode_bytes(&transaction_hash),
      "blockNumber":"0x1",
      "cumulativeGasUsed": "0x0",
      "transactionIndex": "0x0",
      "effectiveGasPrice": "0x0",
      "from": encode_bytes(&[0; 20]),
      "transactionHash": encode_bytes(&transaction_hash),
      "status":"0x1",
      "logs": [],
      "gasUsed":"0x0",
      "gasLimit": "0x0"
    })))
}

pub async fn get_transaction_receipt(transaction_hash: [u8; 32]) -> Result<ResponseValue> {
    Ok(ResponseValue::Value(json!({
      "blockHash":encode_bytes(&transaction_hash),
      "blockNumber":"0x1",
      "cumulativeGasUsed": "0x0",
      "transactionIndex": "0x0",
      "effectiveGasPrice": "0x0",
      "gasLimit": "0x0",
      "transactionHash": transaction_hash,
      "status":"0x1",
      "logs": [],
      "gasUsed":"0x0",
    })))
}
pub async fn get_transaction_count(pool: PgPool, address: [u8; 20]) -> Result<ResponseValue> {
    Ok(ResponseValue::Number(U256::from::<i64>(
        db::get_transaction_count_by_address(&pool, address).await?,
    )))
}
pub async fn get_code(_block_hash: [u8; 20]) -> Result<ResponseValue> {
    Ok(ResponseValue::Value(json!("0x")))
}
pub async fn get_block_by_number(pool: PgPool, block_number: i64) -> Result<ResponseValue> {
    if let Ok(TransactionSignedRow(_, transaction)) =
        db::get_transaction_by_id(&pool, block_number).await
    {
        Ok(ResponseValue::Value(json!({
           "hash": encode_bytes(&transaction.hash().to_vec()),
            "parentHash":encode_bytes(&[0; 32].to_vec()),
            "number": encode_amount(0u32.into()),
            "miner": encode_bytes(&[0; 32].to_vec()),
            "extraData": encode_bytes(&vec![]),
            "gasLimit": encode_amount(0u32.into()),
            "gasUsed": encode_amount(0u32.into()),
            "timestamp": encode_amount(0u32.into()),
            "transactions": vec![encode_bytes(&transaction.hash().to_vec())],
        })))
    } else {
        let transactions: Vec<()> = vec![];
        Ok(ResponseValue::Value(json!({
           "hash": encode_bytes(&[0; 32]),
            "parentHash":encode_bytes(&[0; 32].to_vec()),
            "number": encode_amount(0u32.into()),
            "miner": encode_bytes(&[0; 32].to_vec()),
            "extraData": encode_bytes(&vec![]),
            "gasLimit": encode_amount(0u32.into()),
            "gasUsed": encode_amount(0u32.into()),
            "timestamp": encode_amount(0u32.into()),
            "transactions": transactions,
        })))
    }
}
pub async fn get_block_by_hash(pool: PgPool, block_hash: [u8; 32]) -> Result<ResponseValue> {
    if let Ok(TransactionSignedRow(_, transaction)) =
        db::get_transaction_by_hash(&pool, block_hash).await
    {
        Ok(ResponseValue::Value(json!({
           "hash": encode_bytes(&transaction.hash().to_vec()),
            "parentHash":encode_bytes(&[0; 32].to_vec()),
            "number": encode_amount(0u32.into()),
            "miner": encode_bytes(&[0; 32].to_vec()),
            "extraData": encode_bytes(&vec![]),
            "gasLimit": encode_amount(0u32.into()),
            "gasUsed": encode_amount(0u32.into()),
            "timestamp": encode_amount(0u32.into()),
            "transactions": vec![encode_bytes(&transaction.hash().to_vec())],
        })))
    } else {
        let transactions: Vec<()> = vec![];
        Ok(ResponseValue::Value(json!({
           "hash": encode_bytes(&[0; 32]),
            "parentHash":encode_bytes(&[0; 32].to_vec()),
            "number": encode_amount(0u32.into()),
            "miner": encode_bytes(&[0; 32].to_vec()),
            "extraData": encode_bytes(&vec![]),
            "gasLimit": encode_amount(0u32.into()),
            "gasUsed": encode_amount(0u32.into()),
            "timestamp": encode_amount(0u32.into()),
            "transactions": transactions,
        })))
    }
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

pub async fn block_number(pool: PgPool) -> Result<ResponseValue> {
    let transaction_count = db::get_transaction_count(&pool).await.unwrap_or(0);

    Ok(ResponseValue::Number(U256::from::<u64>(
        (transaction_count).try_into().unwrap(),
    )))
}

pub async fn get_balance(pool: PgPool, address: [u8; 20]) -> Result<ResponseValue> {
    let balance = db::get_balance(&pool, address).await.unwrap_or(0);
    Ok(ResponseValue::Number(scale_up(balance)))
}
