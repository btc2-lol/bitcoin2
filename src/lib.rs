use axum::body::{Body, Bytes};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use k256::ecdsa::VerifyingKey;
use k256::ecdsa::Signature;
use k256::ecdsa::RecoveryId;
use axum::extract::FromRequestParts;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use axum::routing::post;
use axum::{
    http::{header::HeaderName, HeaderMap},
    response::Html,
    routing::get,
    Router,
};
mod crypto;

pub fn app() -> Router {
    Router::new().route("/transactions", post(handler))
}

async fn handler(headers: HeaderMap, message: Bytes) -> impl IntoResponse {
    let public_key = public_key_from_headers(&headers, &message);
    println!("{}", message.len());
    println!("{:?}", public_key);
    StatusCode::OK
}

fn public_key_from_headers(headers: &HeaderMap, message: &Bytes) -> [u8; 33] {
    let auth_header_name = HeaderName::from_static("authorization");
    let signature = BASE64_STANDARD.decode(
        headers.get(auth_header_name).unwrap()
    ).unwrap().as_slice().try_into().unwrap();
    crypto::recover_public_key(
        message,
        signature,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::connect_info::MockConnectInfo,
        http::{self, Request, StatusCode},
    };
    use http_body_util::BodyExt; // for `collect`
    use std::net::SocketAddr;
    use k256::{ecdsa::{SigningKey, signature::Signer}};

    use tokio::net::TcpListener;
    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;
    use tower::{Service, ServiceExt}; // for `call`, `oneshot`, and `ready`

    #[tokio::test]
    async fn status_ok() {

        use k256::{
            ecdsa::{SigningKey as PrivateKey, Signature, signature::Signer},
        };
        use crate::tests::http::HeaderValue;
        use rand_core::OsRng;
        
        let private_key = PrivateKey::random(&mut OsRng);
        let message = vec![1,2,3];
        
        ;
        // let signature: Signature = signing_key.sign(&message);
        // let bytes: [u8; 64] = signature.to_bytes().try_into().unwrap();
        println!("{:?}", private_key.verifying_key().to_sec1_bytes());
        // println!("{:?}", signature.to_bytes().as_bytes());
        let  authorization = BASE64_STANDARD.encode(crypto::sign(private_key, &message));
        let request = Request::builder()
            .method("POST")
            .header("content-type", "application/octet-stream")
            .uri("/transactions")
            .header("Authorization", authorization)
            .body(Body::from(vec![1, 2, 3]))
            .unwrap();

        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.len(), 0usize);
    }
}
