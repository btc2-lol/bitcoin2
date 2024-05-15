use crate::{
    db::{
        get_last_block_timestamp, get_transactions_by_block_number, insert_block,
        update_transactions_block_number,
    },
    error::Result,
};
use digest::Digest;
use reth_primitives::TransactionSigned;
use sha2::Sha256;

use sqlx::PgPool;

use sqlx::types::time::{OffsetDateTime, PrimitiveDateTime};
use tokio::{
    time,
    time::{sleep_until, Duration, Instant},
};

const BLOCK_TIME: tokio::time::Duration = tokio::time::Duration::from_secs(1);

pub async fn start(pool: PgPool) -> Result<()> {
    let last_block_timestamp = get_last_block_timestamp(&pool).await?;
    let next_block_timestamp = datetime_to_tokio_instant(last_block_timestamp) + BLOCK_TIME;

    sleep_until(next_block_timestamp).await;
    let mut ticker = time::interval(BLOCK_TIME);

    loop {
        ticker.tick().await;
        add_block(pool.clone()).await?;
    }
}

async fn add_block(pool: PgPool) -> Result<()> {
    let mut tx = pool.clone().begin().await?;
    let proposed_transactions = get_transactions_by_block_number(&mut *tx, None).await?;
    if proposed_transactions.len() == 0 {
        return Ok(());
    }
    let transaction_ids: Vec<i64> = proposed_transactions.iter().map(|t| t.0).collect();
    let block_number = insert_block(
        &mut *tx,
        block_hash(proposed_transactions.into_iter().map(|t| t.1).collect()),
    )
    .await?;
    update_transactions_block_number(&mut *tx, transaction_ids, block_number).await?;
    tx.commit().await?;
    Ok(())
}

fn block_hash(signed_transations: Vec<TransactionSigned>) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for signed_transaction in signed_transations.iter() {
        println!("hashing: {}", signed_transaction.hash());
        hasher.update(&signed_transaction.hash())
    }
    hasher.finalize().into()
}

fn datetime_to_tokio_instant(datetime: PrimitiveDateTime) -> Instant {
    let now_odt = OffsetDateTime::now_utc();
    let now = PrimitiveDateTime::new(now_odt.date(), now_odt.time());

    let duration = if datetime > now {
        datetime - now
    } else {
        now - datetime
    };

    let secs = duration.whole_seconds();
    let nanos = duration.subsec_nanoseconds();

    if datetime > now {
        Instant::now() + Duration::new(secs as u64, nanos as u32)
    } else {
        Instant::now() - Duration::new(secs as u64, nanos as u32)
    }
}

#[cfg(test)]
mod tests {
    use crate::{app, constants::LAST_LEGACY_BLOCK_NUMBER, db::get_last_block_number, evm::Evm};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use sqlx::PgPool;
    use tower::ServiceExt;

    #[sqlx::test]
    async fn add_block(pool: PgPool) -> sqlx::Result<()> {
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
        super::add_block(pool.clone()).await.unwrap();
        assert_eq!(
            get_last_block_number(&pool.clone()).await.unwrap(),
            LAST_LEGACY_BLOCK_NUMBER + 1
        );
        Ok(())
    }
}
