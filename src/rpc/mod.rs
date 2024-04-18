mod error;

use crate::evm;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract, response::IntoResponse, Json};
use reth_primitives::U256;
use serde_json::{json, Value};

use error::*;
#[derive(serde::Deserialize, Debug)]
pub struct JsonRpcRequest {
    id: Value,
    method: String,
    params: Vec<serde_json::Value>,
}

pub const DEFAULT_GAS_LIMIT: i64 = 21000;

pub trait IntoJson {
    fn into_json(self) -> Value;
}

impl IntoJson for Value {
    fn into_json(self) -> Self {
        self
    }
}

impl IntoJson for i64 {
    fn into_json(self) -> Value {
        format!(
            "0x{}",
            hex::encode(self.to_be_bytes()).trim_start_matches('0')
        )
        .into()
    }
}

impl IntoJson for i32 {
    fn into_json(self) -> Value {
        format!(
            "0x{}",
            hex::encode(self.to_be_bytes()).trim_start_matches('0')
        )
        .into()
    }
}

impl IntoJson for u64 {
    fn into_json(self) -> Value {
        format!(
            "0x{}",
            hex::encode(self.to_be_bytes()).trim_start_matches('0')
        )
        .into()
    }
}

impl IntoJson for Vec<u8> {
    fn into_json(self) -> Value {
        format!("0x{}", hex::encode(self)).into()
    }
}

impl IntoJson for U256 {
    fn into_json(self) -> Value {
        format!(
            "0x{}",
            hex::encode(self.to_be_bytes_vec()).trim_start_matches('0')
        )
        .into()
    }
}

pub async fn handler(extract::Json(request): extract::Json<JsonRpcRequest>) -> impl IntoResponse {
    println!("{:?}", request);
    let result = match request.method.as_ref() {
        "eth_blockNumber" => Ok(block_number(&request.params).into_json()),
        "eth_call" => Ok(call(&request.params).into_json()),
        "eth_chainId" => Ok(chain_id(&request.params).into_json()),
        "eth_estimateGas" => Ok(estimate_gas(&request.params).into_json()),
        "eth_gasPrice" => Ok(gas_price(&request.params).into_json()),
        "eth_getBalance" => Ok(get_balance(&request.params).into_json()),
        "eth_getBlockByHash" => Ok(get_block_by_hash(&request.params).into_json()),
        "eth_getBlockByNumber" => Ok(get_block_by_number(&request.params).into_json()),
        "eth_getCode" => Ok(get_code(&request.params).into_json()),
        "eth_getTransactionCount" => Ok(get_transaction_count(&request.params).into_json()),
        "eth_getTransactionReceipt" => Ok(get_transaction_receipt(&request.params).into_json()),
        "eth_sendRawTransaction" => Ok(send_raw_transaction(&request.params).into_json()),

        _ => Err(Error {
            code: -32601,
            message: format!("Unsupported method [\"{}\"]", request.method),
        }),
    };
    println!("{:?}", result);

    response(request.id, result)
}

pub fn response(request_id: Value, response: std::result::Result<Value, Error>) -> Json<Value> {
    match response {
        Ok(result) => json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "result": result
        })
        .into(),
        Err(error) => json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": error.code,
                "message": error.message,
            }
        })
        .into(),
    }
}
pub fn chain_id(_params: &Vec<Value>) -> impl IntoJson {
    203i64
}
pub fn send_raw_transaction(params: &Vec<Value>) -> impl IntoJson {
    let t = evm::SignedTransaction::decode(&parse_bytes(&params[0]).unwrap());
    println!("{:?}", t);
    vec![0u8; 32]
}

pub fn parse_bytes(value: &Value) -> Result<Vec<u8>> {
    let hex_string = value.as_str().unwrap_or("").trim_start_matches("0x");
    let padded_hex_string = if hex_string.len() % 2 == 0 {
        hex_string.to_string()
    } else {
        format!("0{}", hex_string)
    };

    Ok(hex::decode(padded_hex_string).or(Err(PARSE_ERROR.clone()))?)
}

pub fn get_transaction_receipt(_params: &Vec<Value>) -> impl IntoJson {
    json!("")
}
pub fn get_transaction_count(_params: &Vec<Value>) -> impl IntoJson {
    1
}
pub fn get_code(_params: &Vec<Value>) -> impl IntoJson {
    json!("")
}
pub fn get_block_by_number(_params: &Vec<Value>) -> impl IntoJson {
    json!("")
}
pub fn get_block_by_hash(_params: &Vec<Value>) -> impl IntoJson {
    json!("")
}
pub fn gas_price(_params: &Vec<Value>) -> impl IntoJson {
    0
}

pub fn estimate_gas(_params: &Vec<Value>) -> impl IntoJson {
    json!(DEFAULT_GAS_LIMIT)
}

pub fn call(_params: &Vec<Value>) -> impl IntoJson {
    json!("")
}

pub fn _net_version(_params: &Vec<Value>) -> impl IntoJson {
    2i64
}

pub fn block_number(_params: &Vec<Value>) -> impl IntoJson {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("{:?}", since_the_epoch);
    since_the_epoch.as_secs()
}

pub fn get_balance(_params: &Vec<Value>) -> impl IntoJson {
    evm::scale_up(10000)
}
