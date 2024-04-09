pub mod bitcoin_legacy;
pub mod db;
mod http;
mod protocol;
use protocol::{recover_verifying_key, Message};

use axum::{body::Bytes, extract::State, http::StatusCode, routing::post, Router};
use http::Result;
use sqlx::PgPool;

pub async fn app(pool: PgPool) -> Router {
    Router::new().route("/", post(handler)).with_state(pool)
}

async fn handler(State(_pool): State<PgPool>, mut message: Bytes) -> Result<()> {
    let verifying_key = recover_verifying_key(&mut message)?;
    Message::from_bytes(message)?.execute(verifying_key)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[sqlx::test]
    async fn transfer_by_message(pool: PgPool) {
        let message = hex::decode("00310a34393136386562633832366138326363383463303133393636306439626166623931396136613531643266303162663331363239383936303631653339346430300a303030303030303030303030303030303030303030303030303030303030303030303030303030300a313336323635208472726d44e0a64d178bf30e0919e110b04cecac47de3d54cbc5ad5e78c93cc57d095463b120252e6ab68bd8f9dcb19e8604b7cb420fc69905649d936ba4e417").unwrap();
        let request = Request::builder()
            .method("POST")
            .header("content-type", "application/octet-stream")
            .uri("/")
            .body(Body::from(message))
            .unwrap();

        let response = app(pool).await.oneshot(request).await.unwrap();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        // assert_eq!(body.len(), 0usize);
        println!("{:?}", &body);

        // assert_eq!(response.status(), StatusCode::OK);
    }
}
