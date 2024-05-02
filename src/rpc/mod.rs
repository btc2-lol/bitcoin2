mod error;

use crate::evm;

use crate::evm::{scale_up, Evm};
use axum::{extract, extract::State, response::IntoResponse, Json};
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
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
        if self == 0 {
            return "0x0".into();
        };
        format!(
            "0x{}",
            hex::encode(self.to_be_bytes()).trim_start_matches('0')
        )
        .into()
    }
}

impl IntoJson for i32 {
    fn into_json(self) -> Value {
        if self == 0 {
            return "0x0".into();
        };
        format!(
            "0x{}",
            hex::encode(self.to_be_bytes()).trim_start_matches('0')
        )
        .into()
    }
}

impl IntoJson for u64 {
    fn into_json(self) -> Value {
        if self == 0 {
            return "0x0".into();
        };

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

impl IntoJson for [u8; 32] {
    fn into_json(self) -> Value {
        format!("0x{}", hex::encode(self)).into()
    }
}

impl IntoJson for U256 {
    fn into_json(self) -> Value {
        if self == U256::ZERO {
            return "0x0".into();
        };

        format!(
            "0x{}",
            hex::encode(self.to_be_bytes_vec()).trim_start_matches('0')
        )
        .into()
    }
}
//
pub async fn handler(
    State(evm): State<Evm>,
    extract::Json(request): extract::Json<JsonRpcRequest>,
) -> impl IntoResponse {
    println!("{:?}", request);
    let result = match request.method.as_ref() {
        "eth_blockNumber" => Ok(block_number(evm, &request.params).await.into_json()),
        "eth_call" => Ok(call(&request.params).into_json()),
        "eth_chainId" => Ok(chain_id(&request.params).into_json()),
        "eth_estimateGas" => Ok(estimate_gas(&request.params).into_json()),
        "eth_gasPrice" => Ok(gas_price(&request.params).into_json()),
        "eth_getBalance" => Ok(get_balance(evm, &request.params).await.into_json()),
        "eth_getBlockByHash" => Ok(get_block_by_hash(evm, &request.params).await.into_json()),
        "eth_getBlockByNumber" => Ok(get_block_by_number(evm, &request.params).await.into_json()),
        "eth_getCode" => Ok(get_code(&request.params).into_json()),
        "eth_getTransactionCount" => Ok(get_transaction_count(evm, &request.params)
            .await
            .into_json()),
        "eth_getTransactionReceipt" => Ok(get_transaction_receipt(&request.params).into_json()),
        "eth_sendRawTransaction" => {
            Ok(send_raw_transaction(evm, &request.params).await.into_json())
        }

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
    178i64
}
pub async fn send_raw_transaction(evm: Evm, params: &Vec<Value>) -> impl IntoJson {
    let transaction = evm::SignedTransaction::decode(&parse_bytes(&params[0]).unwrap()).unwrap();

    evm.run_transaction(transaction).await.unwrap()
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

pub fn parse_i64(value: &Value) -> i64 {
    BigUint::from_bytes_be(&parse_bytes(value).unwrap())
        .to_i64()
        .unwrap()
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

pub fn get_transaction_receipt(params: &Vec<Value>) -> impl IntoJson {
    json!({
      "blockHash":"0x0000000000000000000000000000000000000000000000000000000000000001",
      "blockNumber":"0x1",
      "cumulativeGasUsed": "0x0",
      "transactionIndex": "0x0",
      "effectiveGasPrice": "0x0",
      "transactionHash": params[0],
      "status":"0x1",
      "logs": [],
      "gasUsed":"0x0",
    })
}
pub async fn get_transaction_count(evm: Evm, params: &Vec<Value>) -> impl IntoJson {
    evm.get_transaction_count_by_address(parse_bytes(&params[0]).unwrap().try_into().unwrap())
        .await
}
pub fn get_code(_params: &Vec<Value>) -> impl IntoJson {
    json!("0x")
}
pub async fn get_block_by_number(evm: Evm, params: &Vec<Value>) -> impl IntoJson {
    if let Some(hash) = evm.get_transaction_hash(parse_i64(&params[0])).await {
        let params = vec![
            Value::from(encode_bytes(&hash.to_vec())),
            Value::from(false),
        ];
        get_block_by_hash(evm, &params).await
    } else {
        get_block_by_hash(evm, &vec![Value::from(encode_bytes(&[0; 32].to_vec()))]).await
    }
}
pub async fn get_block_by_hash(_evm: Evm, params: &Vec<Value>) -> impl IntoJson {
    json!({
       "hash": params[0].clone(),
        "parentHash":encode_bytes(&[0; 32].to_vec()),
        "number": encode_amount(0u32.into()),
        "miner": encode_bytes(&[0; 32].to_vec()),
        "extraData": encode_bytes(&vec![]),
        "gasLimit": encode_amount(0u32.into()),
        "gasUsed": encode_amount(0u32.into()),
        "timestamp": encode_amount(0u32.into()),
        "transactions": vec![params[0].clone()],
    })
}
pub fn gas_price(_params: &Vec<Value>) -> impl IntoJson {
    0
}

pub fn estimate_gas(_params: &Vec<Value>) -> impl IntoJson {
    json!(DEFAULT_GAS_LIMIT)
}

pub fn call(_params: &Vec<Value>) -> impl IntoJson {
    vec![]
}

pub fn _net_version(_params: &Vec<Value>) -> impl IntoJson {
    2i64
}

pub async fn block_number(_evm: Evm, _params: &Vec<Value>) -> impl IntoJson {
    use std::time::{SystemTime, UNIX_EPOCH};

    let start = SystemTime::now();
    let timestamp = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    json!(timestamp)
}

pub async fn get_balance(evm: Evm, params: &Vec<Value>) -> impl IntoJson {
    let balance = evm
        .get_balance(parse_bytes(&params[0]).unwrap().try_into().unwrap())
        .await
        .unwrap_or(0);
    scale_up(balance)
}
