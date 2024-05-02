use crate::{db::get_last_block_timestamp, error::Result};
use crate::{db::get_transactions_by_block_number, evm::SignedTransaction};
use crate::db::update_transactions_block_number;
use sha2::Sha256;
use digest::Digest;
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::{
    time,
    time::{sleep_until, Instant},
};
const BLOCK_TIME: tokio::time::Duration = tokio::time::Duration::from_secs(1);

pub async fn start(pool: PgPool) -> Result<()> {
    let last_block_timestamp = get_last_block_timestamp(&pool).await?;
    let next_block_timestamp =
        unix_timestamp_to_instant(last_block_timestamp.try_into().unwrap()) + BLOCK_TIME;

    sleep_until(next_block_timestamp).await;
    let mut ticker = time::interval(BLOCK_TIME);

    loop {
        ticker.tick().await;
        let proposed_transactions = get_transactions_by_block_number(&pool, None).await?;
        if proposed_transactions.len() > 0 {
            add_block(pool.clone(), proposed_transactions).await;
        }
    }

    Ok(())
}

async fn add_block(pool: PgPool, signed_transactions: Vec<(i64, SignedTransaction)>) -> Result<()> {
    // let proposed_transactions = get_transactions_by_block_number(&pool, None);
    println!("{:?}", signed_transactions);

    let mut tx = pool.begin().await?;
    let transaction_ids = signed_transactions.iter().map(|t| t.0 ).collect();
    let mut hasher = Sha256::new();

    for signed_transaction in signed_transactions.iter() {
        hasher.update(&borsh::to_vec(signed_transaction)?)
    };
    let result = hasher.finalize();


    update_transactions_block_number(&mut *tx, transaction_ids, 0).await;
    Ok(())
}

fn unix_timestamp_to_instant(unix_timestamp: u64) -> Instant {
    match SystemTime::now()
        .duration_since(UNIX_EPOCH + std::time::Duration::from_secs(unix_timestamp))
    {
        Ok(duration_since_timestamp) => Instant::now() - duration_since_timestamp,
        Err(e) => Instant::now() + e.duration(),
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        block_producer::SignedTransaction,
        db::{get_or_insert_account_id, insert_transaction},
    };
    use crate::db::get_transactions_by_block_number;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn add_block(pool: PgPool) -> sqlx::Result<()> {
        let signed_transaction = SignedTransaction::decode(&hex::decode("f901e7038082520894000000000000000000000000000000000000000080b90184e60b060d00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000008d416374696f6e3a20557067726164650a44657374696e6174696f6e20436861696e2049443a203230330a496e707574733a0a20202d0a20202020486173683a20343931363865626338323661383263633834633031333936363064396261666239313961366135316432663031626633313632393839363036316533393464300a20202020496e6465783a20300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000411fdf09871abfb171e1613469369beaa593830a79d6567f4a2637a97da02b953dfc68efab3e9ce9ec70ce814259aa8bdcf15853d7e26e854016e177b11f73a32aad00000000000000000000000000000000000000000000000000000000000000820188a002051047bd0fabb9f23d1952ee5bdc6e1adafab29995d8733156068c2c025b29a0453c5adcb7a228a0fb2101a5a18b2ea219bee249542f763826586699409f9b84").unwrap()).unwrap();
        let account_id = get_or_insert_account_id(&pool, signed_transaction.signer())
            .await
            .unwrap();
        insert_transaction(&pool, &signed_transaction, account_id).await.unwrap();
        let proposed_transactions = get_transactions_by_block_number(&pool, None).await.unwrap();
        super::add_block(pool, proposed_transactions)
            .await
            .unwrap();
        Ok(())
    }
}
