use bitcoin2::{
    block_producer,
    constants::{Env, ENV, LETS_ENCRYPT_DOMAINS, LETS_ENCRYPT_EMAILS, MIGRATOR, PORT},
};
use dotenv::dotenv;
use rustls_acme::{caches::DirCache, AcmeConfig};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::Ipv6Addr, path::PathBuf};
use tokio::spawn;
use tokio_stream::StreamExt;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().connect(&database_url).await?;

    MIGRATOR.run(&pool).await?;

    spawn({
        let pool = pool.clone();
        async move {
            block_producer::start(pool.clone()).await.unwrap();
        }
    });
    let addr = (Ipv6Addr::UNSPECIFIED, *PORT);
    let app = bitcoin2::app(pool).await;
    if matches!(*ENV, Env::Production) {
        let mut state = AcmeConfig::new(LETS_ENCRYPT_DOMAINS.clone())
            .contact(LETS_ENCRYPT_EMAILS.iter().map(|e| format!("mailto:{}", e)))
            .cache_option(Some(DirCache::new(PathBuf::from(".ssl"))))
            .directory_lets_encrypt(matches!(*ENV, Env::Production))
            .state();
        let acceptor = state.axum_acceptor(state.default_rustls_config());

        tokio::spawn(async move {
            loop {
                match state.next().await.unwrap() {
                    Ok(ok) => println!("event: {:?}", ok),
                    Err(err) => println!("error: {:?}", err),
                }
            }
        });
        axum_server::bind(addr.into())
            .acceptor(acceptor)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?
    };
    Ok(())
}
