pub mod bitcoin_legacy;
pub mod db;
mod evm;
mod http;
mod rpc;
// mod protocol;

use axum::http::header;

use axum::http::method::Method;

use axum::{http::StatusCode, routing::post, Router};

use sqlx::PgPool;

use rpc::handler;
use tower_http::cors::{Any, CorsLayer};

pub async fn app(_pool: PgPool) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(vec![header::CONTENT_TYPE])
        .allow_methods(vec![Method::POST]);

    Router::new().route("/", post(handler)).layer(cors)
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use axum::{body::Body, http::Request};
    // use http_body_util::BodyExt;
    // use tower::ServiceExt;

    // #[sqlx::test]
    // async fn transfer_by_message(pool: PgPool) {
    //     let message = hex::decode("00310a34393136386562633832366138326363383463303133393636306439626166623931396136613531643266303162663331363239383936303631653339346430300a303030303030303030303030303030303030303030303030303030303030303030303030303030300a313336323635208472726d44e0a64d178bf30e0919e110b04cecac47de3d54cbc5ad5e78c93cc57d095463b120252e6ab68bd8f9dcb19e8604b7cb420fc69905649d936ba4e417").unwrap();
    //     let request = Request::builder()
    //         .method("POST")
    //         .header("content-type", "application/octet-stream")
    //         .uri("/")
    //         .body(Body::from(message))
    //         .unwrap();

    //     let response = app(pool).await.oneshot(request).await.unwrap();
    //     assert_eq!(response.status(), StatusCode::OK);
    //     let body = response.into_body().collect().await.unwrap().to_bytes();
    //     assert_eq!(body.len(), 0usize);
    // }
}
