use super::ResponseValue;

use axum::response::Result;
use reth_primitives::U256;

pub async fn version() -> Result<ResponseValue> {
    Ok(ResponseValue::Number(U256::from::<i64>(2)))
}
