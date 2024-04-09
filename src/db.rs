use axum::http::StatusCode;
use sqlx::{Executor, Postgres};

async fn _transfer<E>(pool: E, from: [u8; 20], to: [u8; 20], amount: i64) -> Result<(), StatusCode>
where
    E: Executor<'static, Database = Postgres>,
{
    sqlx::query("call transfer ($1, $2, $3)")
        .bind(from)
        .bind(to)
        .bind(amount)
        .execute(pool)
        .await
        .unwrap();
    Ok(())
}

pub async fn get_balance<E>(pool: E, address: [u8; 20]) -> Result<i64, StatusCode>
where
    E: Executor<'static, Database = Postgres>,
{
    let balance = sqlx::query_as::<_, (i64,)>("select balance from accounts where address = $1")
        .bind(address)
        .fetch_one(pool)
        .await
        .unwrap()
        .0;
    Ok(balance)
}

#[cfg(test)]
mod tests {
    use super::get_balance;
    use crate::StatusCode;
    use sqlx::{Executor, PgPool, Postgres};
    const ALICE: [u8; 20] = [1u8; 20];
    const BOB: [u8; 20] = [2u8; 20];

    #[sqlx::test]
    async fn transfer(pool: PgPool) -> sqlx::Result<()> {
        seed_account(&pool, ALICE, 1).await.unwrap();
        super::_transfer(&pool, ALICE, BOB, 1).await.unwrap();
        assert_eq!(get_balance(&pool, BOB).await.unwrap(), 1);

        Ok(())
    }

    async fn seed_account<E>(
        pool: E,
        account: [u8; 20],
        starting_balance: i64,
    ) -> Result<(), StatusCode>
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
}
