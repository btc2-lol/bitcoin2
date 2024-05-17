pub mod bitcoin_legacy;
pub mod block_producer;
pub mod constants;
pub mod db;
mod error;
pub mod evm;
mod rpc;

use axum::{
    http::{header, method::Method},
    routing::post,
    Router,
};
use sqlx::PgPool;

use tower_http::cors::{Any, CorsLayer};

pub async fn app(pool: PgPool) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(vec![header::CONTENT_TYPE])
        .allow_methods(vec![Method::POST]);

    Router::new()
        .route("/", post(rpc::handler))
        .layer(cors)
        .with_state(pool)
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
        let _evm: Evm = Evm::new(pool.clone());
        let message = json!({
                "jsonrpc": "2.0",
                "method": "eth_sendRawTransaction",
                "params": ["0xf90227068082520894000000000000000000000000000000000000000080b901c4e60b060d0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000cd416374696f6e3a20557067726164650a44657374696e6174696f6e20436861696e2049443a203230330a44657374696e6174696f6e20416464726573733a203078663230344545353539364341626336456336306535653932466434313245413766383536623632350a496e707574733a0a20202d0a20202020486173683a20343931363865626338323661383263633834633031333936363064396261666239313961366135316432663031626633313632393839363036316533393464300a20202020496e6465783a203000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004120239ff874c5e9bcdccf84c0e68333d0337f43847d54d56a3b1e339b4b59a975fb59d0741f8857b9467d00ca418d40f3a7f02e4fa4cbd68daf5c29163310724fdf00000000000000000000000000000000000000000000000000000000000000820188a0af7479b422eefc7e6f3051922e63ceb5330fdd08da66c22d5619f2ba514e24faa01ff98a79df5b2b48007e99e810e0d1d6aeed931993401880efef5dae7f46d790"],
                // "params": ["0xf90227028082520894000000000000000000000000000000000000000080b901c4e60b060d0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000cd416374696f6e3a20557067726164650a44657374696e6174696f6e20436861696e2049443a203230330a44657374696e6174696f6e20416464726573733a203078663230344545353539364341626336456336306535653932466434313245413766383536623632350a496e707574733a0a20202d0a20202020486173683a20343931363865626338323661383263633834633031333936363064396261666239313961366135316432663031626633313632393839363036316533393464300a20202020496e6465783a203000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004120239ff874c5e9bcdccf84c0e68333d0337f43847d54d56a3b1e339b4b59a975fb59d0741f8857b9467d00ca418d40f3a7f02e4fa4cbd68daf5c29163310724fdf00000000000000000000000000000000000000000000000000000000000000820188a0715b95c32d41bea0b0ac8b50b9ab86871fb8f2ff9bed76b0b6a93b46f230f20ca079172a6a6327b32556bf04e6ed46b998a875480d0e67573bb64964523348f736"],
                "id":1
        });
        let request = Request::builder()
            .method("POST")
            .header("content-type", "application/json")
            .uri("/")
            .body(Body::from(message.to_string()))
            .unwrap();

        let response = app(pool.clone()).await.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            get_balance(
                &pool,
                hex_lit::hex!("f204EE5596CAbc6Ec60e5e92Fd412EA7f856b625").into()
            )
            .await
            .unwrap(),
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

        let response = app(pool.clone()).await.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            get_balance(
                &pool,
                hex_lit::hex!("f204ee5596cabc6ec60e5e92fd412ea7f856b625").into()
            )
            .await
            .unwrap(),
            50000000
        );
        assert_eq!(
            get_balance(
                &pool,
                hex_lit::hex!("3073ac44aA1b95f2fe71Bb2eb36b9CE27892F8ee").into()
            )
            .await
            .unwrap(),
            50000000
        );
        let message = json!({
                "jsonrpc": "2.0",
                "method": "eth_getBlockByNumber",
                "params": ["0x1", false],
                "id":1
        });
        let request = Request::builder()
            .method("POST")
            .header("content-type", "application/json")
            .uri("/")
            .body(Body::from(message.to_string()))
            .unwrap();

        let response = app(pool.clone()).await.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    #[sqlx::test]
    async fn get_transactions(pool: PgPool) -> sqlx::Result<()> {
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

        let _response = app(pool.clone()).await.oneshot(request).await.unwrap();
        let message = json!({
            "jsonrpc": "2.0",
            "method": "btc2_getTransactions",
            "id": null,
            "params": [
                "0xf204ee5596cabc6ec60e5e92fd412ea7f856b625"
            ]
        });
        let request = Request::builder()
            .method("POST")
            .header("content-type", "application/json")
            .uri("/")
            .body(Body::from(message.to_string()))
            .unwrap();
        let response = app(pool).await.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }
}
