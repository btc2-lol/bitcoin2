mod btc2;
mod eth;
mod net;

use axum::{extract, extract::State};

use btc2::*;
use eth::*;
use net::*;

use crate::error::{Error, Result};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use reth_primitives::U256;
use serde_json::{json, Value};
use sqlx::PgPool;

#[derive(serde::Deserialize, Debug)]
pub struct JsonRpcRequest {
    id: Value,
    method: String,
    params: Vec<Value>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ParamValue(Value);

impl TryFrom<&ParamValue> for [u8; 20] {
    type Error = Error;

    fn try_from(request_value: &ParamValue) -> Result<Self> {
        if let Value::String(string) = &request_value.0 {
            Ok(parse_bytes_like(string)?.try_into()?)
        } else {
            Err(Error::ParseError(format!(
                "Expected 20 byte length array 4"
            )))
        }
    }
}
impl TryFrom<&ParamValue> for i64 {
    type Error = Error;

    fn try_from(request_value: &ParamValue) -> Result<Self> {
        if let Value::String(string) = &request_value.0 {
            Ok(parse_i64(string)?)
        } else {
            Err(Error::ParseError(format!("Expected i64")))
        }
    }
}
#[derive(Debug)]
enum BlockTag {
    Latest,
    Earliest,
    Pending,
    Safe,
    Finalized,
    Number(i64),
}
impl TryFrom<&ParamValue> for BlockTag {
    type Error = Error;

    fn try_from(request_value: &ParamValue) -> Result<Self> {
        if let Value::String(string) = &request_value.0 {
            match string.as_str() {
                "latest" => Ok(Self::Latest),
                "earliest" => Ok(Self::Earliest),
                "pending" => Ok(Self::Pending),
                "safe" => Ok(Self::Safe),
                "finalized" => Ok(Self::Finalized),
                s if s.starts_with("0x") => {
                    // Use the existing implementation to convert the hexadecimal string to i64
                    let number = i64::try_from(request_value)?;
                    Ok(Self::Number(number))
                },
                _ => Err(Error::ParseError(format!("Invalid block number format or string"))),
            }
        } else {
            Err(Error::ParseError(format!("Expected a string for block number")))
        }
    }
}

impl TryFrom<&ParamValue> for [u8; 32] {
    type Error = Error;

    fn try_from(request_value: &ParamValue) -> Result<Self> {
        Ok(Vec::<u8>::try_from(request_value)?.try_into()?)
    }
}

impl TryFrom<&ParamValue> for Vec<u8> {
    type Error = Error;

    fn try_from(request_value: &ParamValue) -> Result<Self> {
        if let Value::String(string) = &request_value.0 {
            Ok(parse_bytes_like(string)?.try_into()?)
        } else {
            Err(Error::ParseError(format!("Expected 20 byte length array")))
        }
    }
}

pub fn parse_i64(value: &str) -> Result<i64> {
    let bytes = parse_bytes_like(value)?;
    Ok(BigUint::from_bytes_be(&bytes)
        .to_i64()
        .ok_or(Error::ParseError(format!("Expected number got {}", &value)))?)
}

pub fn parse_bytes_like(value: &str) -> Result<Vec<u8>> {
    if value.starts_with("0x") {
        let hex_string = value.trim_start_matches("0x");
        let padded_hex_string = if hex_string.len() % 2 == 0 {
            hex_string.to_string()
        } else {
            format!("0{}", hex_string)
        };
        Ok(hex::decode(padded_hex_string)?)
    } else {
        Err(Error::ParseError(format!(
            "Invalid JSON RPC parameter {}",
            value
        )))
    }
}

pub async fn handler(
    State(pool): State<PgPool>,
    extract::Json(request): extract::Json<JsonRpcRequest>,
) -> axum::response::Result<axum::Json<Value>> {
    // println!("{:?}", request);
    let params: Vec<ParamValue> = request.params.into_iter().map(ParamValue).collect();
    let result = match (request.method.as_ref(), params[..].as_ref()) {
        ("net_version", []) => version().await?,
        ("eth_blockNumber", []) => block_number(pool).await?,
        ("eth_call", [data]) => call(&data.try_into()?).await?,
        ("eth_chainId", []) => chain_id().await?,
        ("eth_estimateGas", [_params]) => estimate_gas().await?,
        ("eth_gasPrice", []) => gas_price().await?,
        ("eth_getBalance", [address, _block_identifier]) => {
            get_balance(pool, address.try_into()?).await?
        }
        ("btc2_getLedger", [address]) => get_transactions(pool, address.try_into()?).await?,
        ("eth_getBlockByHash", [block_hash, _include_full_transactions]) => {
            get_block_by_hash(pool, block_hash.try_into()?).await?
        }
        ("eth_getBlockByNumber", [block_number, _include_full_transactions]) => {
            get_block_by_number(pool, block_number.try_into()?).await?
        }
        ("eth_getCode", [block_hash]) => get_code(block_hash.try_into()?).await?,
        ("eth_getTransactionCount", [address, _block_number]) => {
            get_transaction_count(pool, address.try_into()?).await?
        }
        ("eth_getTransactionByHash", [transaction_hash]) => {
            get_transaction_by_hash(transaction_hash.try_into()?).await?
        }
        ("eth_getTransactionReceipt", [block_hash]) => {
            get_transaction_receipt(block_hash.try_into()?).await?
        }
        ("eth_sendRawTransaction", [raw_transaction]) => {
            send_raw_transaction(pool, raw_transaction.try_into()?).await?
        }
        ("eth_maxPriorityFeePerGas", []) => {
            max_priority_fee_per_gas().await?
        }
        _ => return Err(Error::UnsupportedMethod(request.method.to_string()).into()), // Err(Error {
    };
    // println!("{:?}", &result);

    Ok(axum::Json(json!({
    "jsonrpc": "2.0",
    "id": request.id,
    "result": result.to_value()?
    })))
}
