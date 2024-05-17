use crate::{
    constants::{
        CHAIN_ID, DEFAULT_GAS_LIMIT, LAST_LEGACY_BLOCK_NUMBER, LAST_LEGACY_BLOCK_TIMESTAMP,
        LEGACY_ACCOUNT,
    },
    error::{Error, Result},
    evm::{scale_up, upgrade_by_message::Outpoint, TransactionSigned},
};
use reth_primitives::{Address, Signature, TxKind, TxLegacy};
pub use sqlx::FromRow;
use sqlx::{
    postgres::PgRow, query, query_as, types::time::PrimitiveDateTime, Error::RowNotFound, Executor,
    Postgres, QueryBuilder, Row,
};

pub struct Transaction<'a> {
    inner: sqlx::Transaction<'a, Postgres>,
    pub id: i64,
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
        let id = insert_transaction(&mut *inner, signed_transaction, account_id).await?;

        Ok(Self { inner, id })
    }

    pub async fn upgrade(
        &mut self,
        inputs: Vec<Outpoint>,
        signer: [u8; 20],
        value: i64,
    ) -> Result<()> {
        for input in inputs {
            self.insert_spent_legacy_output(input).await?
        }
        self.transfer(LEGACY_ACCOUNT, signer, value).await
    }

    pub async fn insert_spent_legacy_output(&mut self, vout: Outpoint) -> Result<()> {
        query("INSERT INTO spent_legacy_outputs (transaction_id, hash, index) VALUES ($1, $2, $3)")
            .bind(self.id)
            .bind(vout.hash)
            .bind(vout.index)
            .execute(&mut *self.inner)
            .await?;
        Ok(())
    }

    pub async fn transfer(&mut self, from: [u8; 20], to: [u8; 20], value: i64) -> Result<()> {
        query("CALL transfer ($1, $2, $3, $4)")
            .bind(self.id)
            .bind(from)
            .bind(to)
            .bind(value)
            .execute(&mut *self.inner)
            .await?;
        Ok(())
    }

    pub async fn commit(self) -> Result<i64> {
        self.inner.commit().await?;

        Ok(self.id)
    }
}

pub async fn get_last_block_timestamp<'a, E>(pool: E) -> Result<PrimitiveDateTime>
where
    E: Executor<'a, Database = Postgres>,
{
    let result =
        query_as::<_, (PrimitiveDateTime,)>("SELECT timestamp FROM blocks ORDER BY blocks.number")
            .fetch_one(pool)
            .await;
    if matches!(result, Err(RowNotFound)) {
        return Ok(LAST_LEGACY_BLOCK_TIMESTAMP);
    };

    Ok(result?.0)
}

pub async fn get_last_block_number<'a, E>(pool: E) -> Result<i64>
where
    E: Executor<'a, Database = Postgres>,
{
    let result = query_as::<_, (i64,)>("select number from blocks order by blocks.number")
        .fetch_one(pool)
        .await;
    if matches!(result, Err(sqlx::Error::RowNotFound)) {
        return Ok(LAST_LEGACY_BLOCK_NUMBER);
    };

    Ok(result?.0)
}

pub async fn get_balance<E>(pool: E, address: [u8; 20]) -> Result<i64>
where
    E: Executor<'static, Database = Postgres>,
{
    Ok(query("SELECT balance FROM accounts WHERE address = $1")
        .bind(address)
        .fetch_one(pool)
        .await
        .map(|row| row.get(0))?)
}

pub async fn get_transaction_by_id<'a, E>(
    pool: E,
    transaction_id: i64,
) -> Result<TransactionSignedRow>
where
    E: Executor<'a, Database = Postgres>,
{
    let query = "
        SELECT transactions.*,
        entries.*,
        accounts_to.address as to_address,
        accounts_from.address as from_address
        FROM transactions 
        JOIN entries ON transactions.id = entries.transaction_id
        JOIN accounts accounts_to ON entries.to_id = accounts_to.id
        JOIN accounts accounts_from ON entries.from_id = accounts_from.id
        WHERE transactions.id = $1
    ";


    let transaction_signed_row = query_as::<_, TransactionSignedRow>(query)
        .bind(transaction_id)
        .fetch_one(pool)
        .await?;

    Ok(transaction_signed_row)
}

pub async fn get_transaction_by_hash<'a, E>(pool: E, hash: [u8; 32]) -> Result<TransactionSignedRow>
where
    E: Executor<'a, Database = Postgres>,
{
    let query = "
        SELECT transactions.*,
        entries.*,
        accounts_to.address as to_address,
        accounts_from.address as from_address
        FROM transactions 
        JOIN entries ON transactions.id = entries.transaction_id
        JOIN accounts accounts_to ON entries.to_id = accounts_to.id
        JOIN accounts accounts_from ON entries.from_id = accounts_from.id
        WHERE hash = $1
    ";


    let transaction_signed_row = query_as::<_, TransactionSignedRow>(query)
        .bind(hash)
        .fetch_one(pool)
        .await?;

    Ok(transaction_signed_row)
}

pub async fn get_transactions_by_block_number<'a, E>(
    pool: E,
    block_number: Option<i64>,
) -> Result<Vec<TransactionSignedRow>>
where
    E: Executor<'a, Database = Postgres>,
{
    let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "SELECT transactions.*,
        entries.*,
        accounts_to.address as to_address,
        accounts_from.address as from_address
        FROM transactions 
        JOIN entries ON transactions.id = entries.transaction_id
        JOIN 
            accounts accounts_to ON entries.to_id = accounts_to.id
        JOIN 
            accounts accounts_from ON entries.from_id = accounts_from.id
        ",
    );

    if let Some(block_number) = block_number {
        builder.push(" WHERE block_number = ");
        builder.push_bind(block_number);
    } else {
        builder.push(" WHERE block_number IS NULL");
    }

    Ok(query_as(builder.sql()).fetch_all(pool).await?)
}

pub struct TransactionSignedRow(pub i64, pub TransactionSigned);
impl FromRow<'_, PgRow> for TransactionSignedRow {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let to = if let Some(to) = row.get::<Option<Vec<u8>>, _>("to_address") {
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
    
    println!("get_transactions_by_address");
    let transactions: Vec<TransactionSignedRow> = query_as(
        "SELECT transactions.*,
        entries.*,
        accounts_to.address as to_address,
        accounts_from.address as from_address
        from
         entries
        join transactions on entries.transaction_id = transactions.id
        join accounts accounts_from on entries.from_id = accounts_from.id
        join accounts accounts_to on entries.to_id = accounts_to.id

       where $1 IN (accounts_from.address, accounts_to.address);"
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
    query("INSERT into accounts (address, balance) VALUES ($1, $2)")
        .bind(account)
        .bind(starting_balance)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_block<'a, E: Executor<'a, Database = Postgres>>(
    e: E,
    hash: [u8; 32],
) -> Result<i64> {
    Ok(
        query("INSERT INTO blocks (hash, timestamp) VALUES ($1, NOW()) RETURNING number")
            .bind(hash)
            .fetch_one(e)
            .await
            .map(|row| row.get(0))?,
    )
}

pub async fn insert_transaction<'a, E: Executor<'a, Database = Postgres>>(
    e: E,
    signed_transaction: &TransactionSigned,
    account_id: i64,
) -> Result<i64> {
    let mut signature = Vec::new();
    signed_transaction.signature().encode(&mut signature);
    let record = query("INSERT INTO transactions (hash, account_id, nonce, gas_price, input, signature) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id")
        .bind(signed_transaction.hash().to_vec())
        .bind(account_id)
        .bind(signed_transaction.transaction.nonce() as i64)
        .bind(signed_transaction.transaction.max_fee_per_gas() as i64)
        .bind(signed_transaction.transaction.input().to_vec())
        .bind(signature)
        .fetch_one(e)
        .await;

    Ok(record.map(|row| row.get(0))?)
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
    Ok(query("SELECT COUNT(*) FROM transactions JOIN accounts on transactions.id = accounts.id WHERE accounts.address = $1")
            .bind(address)
            .fetch_one(pool)
            .await?.get(0)
        )
}

pub async fn get_transaction_count<'a, E: Executor<'a, Database = Postgres>>(
    pool: E,
) -> Result<i64> {
    Ok(query("SELECT COUNT(*) FROM transactions")
        .fetch_one(pool)
        .await?
        .get(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    const ALICE: [u8; 20] = [0; 20]; 
    const BOB: [u8; 20] = [1; 20]; 

    #[sqlx::test]
    async fn get_transactions_by_address(pool: PgPool) -> sqlx::Result<()> {
        deposit(
            &pool,
            hex_lit::hex!("f204ee5596cabc6ec60e5e92fd412ea7f856b625").into(),
            100000000,
        )
        .await;
        let transaction_signed = TransactionSigned::decode_rlp_legacy_transaction(&mut &hex::decode("f8690180825208943073ac44aa1b95f2fe71bb2eb36b9ce27892f8ee8806f05b59d3b20000808201b9a0d95066012c1af3689ac24030b965a81211b506022d4db117bf90b4a22ccaf981a03c818c75f0634ee921cbcb290371c5e14e76768db4f18900753dbcce651978eb").unwrap()[..]).unwrap();
        let mut transaction = Transaction::new(&pool,
            &transaction_signed 
        ).await.unwrap();
        transaction.transfer(hex_lit::hex!("f204ee5596cabc6ec60e5e92fd412ea7f856b625").into(), hex_lit::hex!("3073ac44aA1b95f2fe71Bb2eb36b9CE27892F8ee").into(), 1).await.unwrap();
        transaction.commit().await.unwrap(); 

        assert_eq!(super::get_transactions_by_address(&pool, hex_lit::hex!("f204ee5596cabc6ec60e5e92fd412ea7f856b625").into()).await.unwrap().len(), 1);

        Ok(())
    }
}