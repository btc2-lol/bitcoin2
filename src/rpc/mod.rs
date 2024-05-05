mod btc2;
mod eth;
mod net;

use crate::evm::Evm;
use axum::{extract, extract::State};

use btc2::*;
use eth::*;
use net::*;

use reth_primitives::U256;
use serde_json::{json, Value};

#[derive(serde::Deserialize, Debug)]
pub struct JsonRpcRequest {
    id: Value,
    method: String,
    params: Vec<serde_json::Value>,
}

pub enum ResponseValue {
    Number(U256),
    Value(Value),
    Null,
}

impl ResponseValue {
    fn to_value(self) -> crate::error::Result<Value> {
        match self {
            Self::Number(number) => {
                if number == U256::ZERO {
                    return Ok("0x0".into());
                };

                Ok(format!(
                    "0x{}",
                    hex::encode(number.to_be_bytes_vec()).trim_start_matches('0')
                )
                .into())
            }
            Self::Value(value) => Ok(value),
            Self::Null => Ok(json!(null)),
        }
    }
}

pub enum RequestValue {
    Bytes(Vec<u8>),
}
impl TryFrom<&RequestValue> for [u8; 20] {
    type Error = crate::error::Error;

    fn try_from(request_value: &RequestValue) -> crate::error::Result<Self> {
        let RequestValue::Bytes(bytes) = request_value;
        Ok(bytes.to_vec().try_into().unwrap())
    }
}
impl TryFrom<&RequestValue> for [u8; 32] {
    type Error = crate::error::Error;

    fn try_from(request_value: &RequestValue) -> crate::error::Result<Self> {
        let RequestValue::Bytes(bytes) = request_value;
        Ok(bytes.to_vec().try_into().unwrap())
    }
}
impl TryFrom<&RequestValue> for Vec<u8> {
    type Error = crate::error::Error;

    fn try_from(request_value: &RequestValue) -> crate::error::Result<Self> {
        let RequestValue::Bytes(bytes) = request_value;
        Ok(bytes.to_vec().try_into().unwrap())
    }
}

impl TryFrom<&RequestValue> for i64 {
    type Error = crate::error::Error;

    fn try_from(request_value: &RequestValue) -> crate::error::Result<Self> {
        let RequestValue::Bytes(bytes) = request_value;
        Ok(parse_i64(&bytes.to_vec()))
    }
}

fn parse_error(value: String) -> crate::error::Error {
    crate::error::Error::ParseError(format!("Invalid JSON RPC parameter {}", value))
}

fn parse_params(params: Vec<Value>) -> crate::error::Result<Vec<RequestValue>> {
    params
        .iter()
        .map(|value| {
            parse_hex_param(
                value
                    .as_str()
                    .ok_or(crate::error::Error::ParseError(format!(
                        "Expected string got {}",
                        value
                    )))?
                    .to_string(),
            )
        })
        .collect()
}

pub fn parse_hex_param(value: String) -> crate::error::Result<RequestValue> {
    if value.starts_with("0x") {
        let hex_string = value.trim_start_matches("0x");
        let padded_hex_string = if hex_string.len() % 2 == 0 {
            hex_string.to_string()
        } else {
            format!("0{}", hex_string)
        };
        Ok(RequestValue::Bytes(hex::decode(padded_hex_string)?))
    } else {
        Err(parse_error(value))
    }
}
pub async fn handler(
    State(evm): State<Evm>,
    extract::Json(request): extract::Json<JsonRpcRequest>,
) -> axum::response::Result<axum::Json<Value>> {
    println!("{:?}", request);
    let result = match (
        request.method.as_ref(),
        parse_params(request.params)?[..].as_ref(),
    ) {
        ("net_version", []) => version().await?,
        ("eth_blockNumber", [block_id]) => block_number(evm, block_id.try_into()?).await?,
        ("eth_call", [data]) => call(&data.try_into()?).await?,
        ("eth_chainId", []) => chain_id().await?,
        ("eth_estimateGas", []) => estimate_gas().await?,
        ("eth_gasPrice", []) => gas_price().await?,
        ("eth_getBalance", [address]) => get_balance(evm, address.try_into()?).await?,
        ("btc2_getTransactions", [address]) => get_transactions(evm, address.try_into()?).await?,
        ("eth_getBlockByHash", [block_hash]) => {
            get_block_by_hash(evm, block_hash.try_into()?).await?
        }
        ("eth_getBlockByNumber", [block_number]) => {
            get_block_by_number(evm, block_number.try_into()?).await?
        }
        ("eth_getCode", [block_hash]) => get_code(block_hash.try_into()?).await?,
        ("eth_getTransactionCount", [address]) => {
            get_transaction_count(evm, address.try_into()?).await?
        }
        ("eth_getTransactionByHash", [block_hash]) => {
            get_transaction_by_hash(block_hash.try_into()?).await?
        }
        ("eth_getTransactionReceipt", [block_hash]) => {
            get_transaction_receipt(block_hash.try_into()?).await?
        }
        ("eth_sendRawTransaction", [raw_transaction]) => {
            send_raw_transaction(evm, raw_transaction.try_into()?).await?
        }
        _ => return Err(crate::error::Error::UnsupportedMethod(request.method.to_string()).into()), // Err(Error {
    };

    Ok(axum::Json(json!({
    "jsonrpc": "2.0",
    "id": request.id,
    "result": result.to_value()?
    })))
}
