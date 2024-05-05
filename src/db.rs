use crate::{
    error::Result,
    evm::{upgrade_by_message::Outpoint, SignedTransaction},
};

use sqlx::{query, Executor, Postgres, Row};

const LAST_LEGACY_BLOCK_TIMESTAMP: i64 = 1713557133;
pub const LAST_LEGACY_BLOCK_NUMBER: i64 = 83999;

macro_rules! account_id {
    ($last_byte:expr) => {{
        let mut array = [0u8; 20];
        array[19] = $last_byte;
        array
    }};
}

pub const LEGACY_ACCOUNT: [u8; 20] = account_id!(0);

pub struct Transaction<'a> {
    inner: sqlx::Transaction<'a, Postgres>,
    transaction_id: i64,
}
impl Transaction<'_> {
    pub async fn new(
        pool: &sqlx::Pool<Postgres>,
        signed_transaction: &SignedTransaction,
    ) -> Result<Self> {
        let mut inner = pool.begin().await?;

        let account_id = get_or_insert_account_id(&mut *inner, signed_transaction.signer()).await?;
        let transaction_id =
            insert_transaction(&mut *inner, signed_transaction, account_id).await?;
        Ok(Self {
            inner,
            transaction_id,
        })
    }
    pub async fn upgrade(
        &mut self,
        _block_number: i64,
        _transaction_hash: [u8; 32],
        inputs: Vec<Outpoint>,
        signer: [u8; 20],
        amount: i64,
    ) -> Result<()> {
        let _account_id = get_or_insert_account_id(&mut *self.inner, signer).await?;

        for input in inputs {
            self.insert_spent_legacy_output(input).await?
        }
        self.transfer(LEGACY_ACCOUNT, signer, amount).await
    }

    pub async fn insert_spent_legacy_output(&mut self, vout: Outpoint) -> Result<()> {
        query("INSERT INTO spent_legacy_outputs (transaction_id, hash, index) VALUES ($1, $2, $3)")
            .bind(self.transaction_id)
            .bind(vout.hash)
            .bind(vout.index)
            .execute(&mut *self.inner)
            .await?;
        Ok(())
    }

    pub async fn transfer(&mut self, from: [u8; 20], to: [u8; 20], amount: i64) -> Result<()> {
        println!("{:?}", from);
        println!("{:?}", to);
        println!("{:?}", amount);
        query("CALL transfer ($1, $2, $3, $4)")
            .bind(self.transaction_id)
            .bind(from)
            .bind(to)
            .bind(amount)
            .execute(&mut *self.inner)
            .await
            .unwrap();
        Ok(())
    }

    pub async fn commit(self) -> Result<()> {
        Ok(self.inner.commit().await?)
    }
}

pub async fn get_last_block_timestamp<'a, E>(pool: E) -> Result<i64>
where
    E: Executor<'a, Database = Postgres>,
{
    let result = sqlx::query_as::<_, (i64,)>("select timestamp from blocks order by blocks.number")
        .fetch_one(pool)
        .await;
    if matches!(result, Err(sqlx::Error::RowNotFound)) {
        return Ok(LAST_LEGACY_BLOCK_TIMESTAMP);
    };

    Ok(result?.0)
}

pub async fn get_last_block_number<'a, E>(pool: E) -> Result<i64>
where
    E: Executor<'a, Database = Postgres>,
{
    let result = sqlx::query_as::<_, (i64,)>("select number from blocks order by blocks.number")
        .fetch_one(pool)
        .await;
    if matches!(result, Err(sqlx::Error::RowNotFound)) {
        return Ok(LAST_LEGACY_BLOCK_NUMBER);
    };

    Ok(result?.0)
}

pub async fn get_transaction_hashes<'a, E>(pool: E) -> Result<Vec<[u8; 32]>>
where
    E: Executor<'a, Database = Postgres>,
{
    let balance = sqlx::query_as::<_, (Vec<[u8; 32]>,)>("select hash(raw) from transactions")
        .fetch_one(pool)
        .await?
        .0;
    Ok(balance)
}

pub async fn get_balance<E>(pool: E, address: [u8; 20]) -> Result<i64>
where
    E: Executor<'static, Database = Postgres>,
{
    let balance = sqlx::query_as::<_, (i64,)>("select balance from accounts where address = $1")
        .bind(address)
        .fetch_one(pool)
        .await?
        .0;
    Ok(balance)
}

pub async fn get_transactions_by_block_number<'a, E>(
    pool: E,
    block_number: Option<i64>,
) -> Result<Vec<(i64, SignedTransaction)>>
where
    E: Executor<'a, Database = Postgres>,
{
    let transacions_bytes: Vec<(i64, Vec<u8>)> =
        query("select id, raw from transactions where block_number = $1")
            .bind(block_number)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|result| (result.get(0), result.get::<Vec<u8>, _>(1)))
            .collect::<Vec<(i64, Vec<u8>)>>();
    Ok(transacions_bytes
        .into_iter()
        .map(|data| {
            SignedTransaction::decode(&data.1)
                .map_err(|e| crate::error::Error::Error(e.to_string()))
                .map(|t| (data.0, t))
        })
        .collect::<Result<Vec<(i64, SignedTransaction)>>>()
        .map_err(|e| crate::error::Error::Error(e.to_string()))?)
}

pub async fn get_transactions_by_address<'a, E>(
    pool: E,
    address: [u8; 32],
) -> Result<Vec<(i64, SignedTransaction)>>
where
    E: Executor<'a, Database = Postgres>,
{
    let transacions_bytes: Vec<(i64, Vec<u8>)> =
        query("select id, raw from transactions where account_id = $1")
            .bind(address)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|result| (result.get(0), result.get::<Vec<u8>, _>(1)))
            .collect::<Vec<(i64, Vec<u8>)>>();
    Ok(transacions_bytes
        .into_iter()
        .map(|data| {
            SignedTransaction::decode(&data.1)
                .map_err(|e| crate::error::Error::Error(e.to_string()))
                .map(|t| (data.0, t))
        })
        .collect::<Result<Vec<(i64, SignedTransaction)>>>()
        .map_err(|e| crate::error::Error::Error(e.to_string()))?)
}

pub async fn deposit<E>(pool: E, account: [u8; 20], starting_balance: i64) -> Result<()>
where
    E: Executor<'static, Database = Postgres>,
{
    sqlx::query("insert into accounts (address, balance) values ($1, $2)")
        .bind(account)
        .bind(starting_balance)
        .execute(pool)
        .await
        .unwrap();
    Ok(())
}

pub async fn insert_block<'a, E: Executor<'a, Database = Postgres>>(
    e: E,
    hash: [u8; 32],
) -> Result<i64> {
    let record = sqlx::query("INSERT INTO blocks (hash) VALUES ($1) RETURNING id")
        .bind(hash)
        .fetch_one(e)
        .await?;

    let id: i64 = record.get(0);
    Ok(id)
}

pub async fn insert_transaction<'a, E: Executor<'a, Database = Postgres>>(
    e: E,
    signed_transaction: &SignedTransaction,
    account_id: i64,
) -> Result<i64> {
    let record = sqlx::query(
        "INSERT INTO transactions (raw, hash, account_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(borsh::to_vec(&signed_transaction)?)
    .bind(&signed_transaction.hash())
    .bind(account_id)
    .fetch_one(e)
    .await?;

    let id: i64 = record.get(0);
    Ok(id)
}

pub async fn update_transactions_block_number<'a, E>(
    pool: E,
    transaction_ids: Vec<i64>,
    block_number: i64,
) -> Result<()>
where
    E: Executor<'a, Database = Postgres>,
{
    sqlx::query(
        "
    UPDATE transactions
    SET block_number = $1
    WHERE id = ANY($2)
",
    )
    .bind(block_number)
    .bind(transaction_ids)
    .execute(pool)
    .await?;
    Ok(())
}

// pub async fn get_account_id<'a, E: Executor<'a, Database = Postgres>>(
//     e: E,
//     address: [u8; 20],
// ) -> Result<Option<i64>> {
//     let result = sqlx::query("SELECT account_id from accounts where address=$1")
//         .bind(address)
//         .fetch_one(e)
//         .await
//         .map_err(|_e| {
//             println!("{}", _e);
//             StatusCode::INTERNAL_SERVER_ERROR
//         })?;
//     if matches!(result, Err(sqlx::Error::RowNotFound)) {
//         return Ok(LAST_LEGACY_BLOCK_TIMESTAMP);
//     };
//     Ok(result.get(0))
// }

pub async fn get_or_insert_account_id<'a, E: Executor<'a, Database = Postgres>>(
    e: E,
    address: [u8; 20],
) -> Result<i64> {
    let result = sqlx::query("SELECT select_or_insert_account($1)")
        .bind(address)
        .fetch_one(e)
        .await?;
    Ok(result.get(0))
}

pub async fn get_transaction_count_by_address<E>(pool: E, address: [u8; 20]) -> Result<i64>
where
    E: Executor<'static, Database = Postgres>,
{
    let result = sqlx::query("SELECT COUNT(*) FROM transactions JOIN accounts on transactions.id = accounts.id WHERE accounts.address = $1")
            .bind(address)
            .fetch_one(pool)
            .await?;

    Ok(result.get(0))
}

pub async fn get_transaction_hash<E>(pool: E, id: i64) -> Result<[u8; 32]>
where
    E: Executor<'static, Database = Postgres>,
{
    let result = sqlx::query("SELECT hash FROM transactions WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(result.get(0))
}

pub async fn get_transaction_count<E>(pool: E) -> Result<i64>
where
    E: Executor<'static, Database = Postgres>,
{
    let result = sqlx::query("SELECT COUNT(*) FROM transactions")
        .fetch_one(pool)
        .await?;

    Ok(result.get(0))
}
