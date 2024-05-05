use crate::{
    constants::{CHAIN_ID, DEFAULT_GAS_LIMIT},
    error::{Error, Result},
    evm::{scale_down, scale_up, upgrade_by_message::Outpoint, TransactionSigned},
};
use reth_primitives::{Address, Signature, TxKind, TxLegacy};
pub use sqlx::FromRow;
use sqlx::{postgres::PgRow, query, query_as, Executor, Postgres, Row};

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
        signed_transaction: &TransactionSigned,
    ) -> Result<Self> {
        let mut inner = pool.begin().await?;

        let account_id = get_or_insert_account_id(
            &mut *inner,
            signed_transaction
                .recover_signer()
                .map(|signer| -> Result<[u8; 20]> { signer.try_into().map_err(Error::from) })
                .ok_or(Error::InvalidSignature)??,
        )
        .await?;
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
        for input in inputs {
            self.insert_spent_legacy_output(input).await?
        }
        self.transfer(LEGACY_ACCOUNT, signer, amount).await
    }

    pub fn id_as_hash(&self) -> [u8; 32] {
        let mut array = [0u8; 32];
        array[24..].copy_from_slice(&self.transaction_id.to_be_bytes());

        array
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
    let hashes = query_as("select transactions.* from transactions")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|signed_transaction: TransactionSignedRow| {
            signed_transaction.1.hash().to_vec().try_into().unwrap()
        })
        .collect::<Vec<[u8; 32]>>();
    Ok(hashes)
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
) -> Result<Vec<TransactionSignedRow>>
where
    E: Executor<'a, Database = Postgres>,
{
    Ok(
        query_as("select * from transactions where block_number = $1")
            .bind(block_number)
            .fetch_all(pool)
            .await?,
    )
}

pub struct TransactionSignedRow(pub i64, pub TransactionSigned);
impl FromRow<'_, PgRow> for TransactionSignedRow {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let to = if let Some(to) = row.get::<Option<Vec<u8>>, _>("_to") {
            TxKind::Call(Address::new(to.try_into().unwrap()))
        } else {
            TxKind::Create
        };
        let signature_bytes = row.get::<Vec<u8>, _>("signature");
        let signature = Signature::decode(&mut &signature_bytes[..]).unwrap();

        Ok(Self(
            row.get::<i64, _>("id"),
            TransactionSigned::from_transaction_and_signature(
                reth_primitives::transaction::Transaction::Legacy(TxLegacy {
                    chain_id: Some(CHAIN_ID as u64),
                    gas_limit: DEFAULT_GAS_LIMIT.try_into().unwrap(),
                    gas_price: row.get::<i64, _>("gas_price").try_into().unwrap(),
                    to,
                    value: scale_up(row.get::<i64, _>("value")),
                    input: row.get::<Vec<u8>, _>("input").into(),
                    nonce: row.get::<i64, _>("nonce") as u64,
                }),
                signature,
            ),
        ))
    }
}

pub async fn get_transactions_by_address<'a, E>(
    pool: E,
    address: [u8; 20],
) -> Result<Vec<TransactionSigned>>
where
    E: Executor<'a, Database = Postgres>,
{
    let transactions: Vec<TransactionSignedRow> = query_as(
        "select
            transactions.*
        from transactions
        join accounts on transactions.account_id = accounts.id
        join entries
        on transactions.account_id = entries.creditor_id
         or transactions.account_id = entries.debtor_id

       where accounts.address = $1;
        ",
    )
    .bind(address)
    .fetch_all(pool)
    .await?
    .into();
    Ok(transactions.into_iter().map(|t| t.1).collect())
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
    signed_transaction: &TransactionSigned,
    account_id: i64,
) -> Result<i64> {
    let mut signature = Vec::new();
    signed_transaction.signature().encode(&mut signature);
    let record = query("INSERT INTO transactions (account_id, nonce, gas_price, _to, value, input, signature) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id")
    .bind(account_id)
    .bind(signed_transaction.transaction.nonce() as i64)
    .bind(signed_transaction.transaction.max_fee_per_gas() as i64)
    .bind(signed_transaction.transaction.to().map(|to| to.to_vec()))
    .bind(scale_down(signed_transaction.transaction.value()))
    .bind(signed_transaction.transaction.input().to_vec())
    .bind(signature)
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
