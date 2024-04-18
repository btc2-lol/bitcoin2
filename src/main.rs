use dotenv::dotenv;
use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use std::{env, net::Ipv4Addr};

static MIGRATOR: Migrator = sqlx::migrate!();
macro_rules! account_id {
    ($last_byte:expr) => {{
        let mut array = [0u8; 20];
        array[19] = $last_byte;
        array
    }};
}

const _LEGACY_ACCOUNT: [u8; 20] = account_id!(0x00);

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port = env::var("PORT").and_then(|port| Ok(port.parse().unwrap_or(3000)))?;
    let pool = PgPoolOptions::new().connect(&database_url).await?;
    MIGRATOR.run(&pool).await?;
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), port)).await?;
    axum::serve(listener, bitcoin2::app(pool).await).await?;

    Ok(())
}
