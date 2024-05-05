pub mod bitcoin_legacy;
pub mod block_producer;
pub mod db;
mod error;
pub mod evm;
mod rpc;

use crate::evm::Evm;
use axum::{
    http::{header, method::Method},
    routing::post,
    Router,
};

use tower_http::cors::{Any, CorsLayer};

pub async fn app(evm: Evm) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(vec![header::CONTENT_TYPE])
        .allow_methods(vec![Method::POST]);

    Router::new()
        .route("/", post(rpc::handler))
        .layer(cors)
        .with_state(evm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::get_balance, evm::Evm};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use sqlx::PgPool;
    use tower::ServiceExt;

    #[sqlx::test]
    async fn upgrade_by_message(pool: PgPool) -> sqlx::Result<()> {
        let evm: Evm = Evm::new(pool.clone());
        let message = json!({
                "jsonrpc": "2.0",
                "method": "eth_sendRawTransaction",
                "params": ["0xf901e7038082520894000000000000000000000000000000000000000080b90184e60b060d00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000008d416374696f6e3a20557067726164650a44657374696e6174696f6e20436861696e2049443a203230330a496e707574733a0a20202d0a20202020486173683a20343931363865626338323661383263633834633031333936363064396261666239313961366135316432663031626633313632393839363036316533393464300a20202020496e6465783a20300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000411fdf09871abfb171e1613469369beaa593830a79d6567f4a2637a97da02b953dfc68efab3e9ce9ec70ce814259aa8bdcf15853d7e26e854016e177b11f73a32aad00000000000000000000000000000000000000000000000000000000000000820188a002051047bd0fabb9f23d1952ee5bdc6e1adafab29995d8733156068c2c025b29a0453c5adcb7a228a0fb2101a5a18b2ea219bee249542f763826586699409f9b84"],
                "id":1
        });
        let request = Request::builder()
            .method("POST")
            .header("content-type", "application/json")
            .uri("/")
            .body(Body::from(message.to_string()))
            .unwrap();

        let response = app(evm).await.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            get_balance(
                &pool,
                hex_lit::hex!("f204EE5596CAbc6Ec60e5e92Fd412EA7f856b625").into()
            )
            .await?,
            136265
        );
        Ok(())
    }
    #[sqlx::test]
    async fn transfer(pool: PgPool) -> sqlx::Result<()> {
        let evm: Evm = Evm::new(pool.clone());
        evm.deposit(
            hex_lit::hex!("f204ee5596cabc6ec60e5e92fd412ea7f856b625").into(),
            100000000,
        )
        .await;

        let message = json!({
                "jsonrpc": "2.0",
                "method": "eth_sendRawTransaction",
                "params": ["0xf8690180825208943073ac44aa1b95f2fe71bb2eb36b9ce27892f8ee8806f05b59d3b20000808201b9a0d95066012c1af3689ac24030b965a81211b506022d4db117bf90b4a22ccaf981a03c818c75f0634ee921cbcb290371c5e14e76768db4f18900753dbcce651978eb"],
                "id":1
        });
        let request = Request::builder()
            .method("POST")
            .header("content-type", "application/json")
            .uri("/")
            .body(Body::from(message.to_string()))
            .unwrap();

        let response = app(evm).await.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            get_balance(
                &pool,
                hex_lit::hex!("f204ee5596cabc6ec60e5e92fd412ea7f856b625").into()
            )
            .await?,
            50000000
        );
        assert_eq!(
            get_balance(
                &pool,
                hex_lit::hex!("3073ac44aA1b95f2fe71Bb2eb36b9CE27892F8ee").into()
            )
            .await?,
            50000000
        );
        Ok(())
    }
}
