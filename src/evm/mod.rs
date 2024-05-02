pub mod postgres;
pub mod rocksdb;
#[cfg(test)]
mod test_utils;
pub mod transaction;
pub mod upgrade_by_message;

use crate::db::{
    deposit, get_balance, get_transaction_count, get_transaction_count_by_address,
    get_transaction_hash,
};
use anyhow::anyhow;
use reth_primitives::U256;

use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
pub use transaction::SignedTransaction;
use upgrade_by_message::UpgradeByMessage;

const SYSTEM_ADDRESS: [u8; 20] = [0; 20];
// ethers.FunctionFragment.getSelector('upgradeByMessage', ['string', 'bytes'])
const UPGRADE_BY_MESSAGE: [u8; 4] = *b"\xe6\x0b\x06\x0d";
#[derive(Clone)]
pub struct Evm {
    db: Arc<Mutex<postgres::PgDb>>,
}

impl Evm {
    pub fn new(pool: PgPool) -> Self {
        let postgres_db = postgres::PgDb::new(pool);

        Self {
            db: Arc::new(Mutex::new(postgres_db)),
        }
    }
    pub async fn get_balance(&self, address: [u8; 20]) -> Option<i64> {
        let db = self.db.lock().await;
        get_balance(&db.pool, address).await.ok()
    }

    pub async fn get_transaction_hash(&self, id: i64) -> Option<[u8; 32]> {
        let db = self.db.lock().await;
        get_transaction_hash(&db.pool, id).await.ok()
    }

    pub async fn get_transaction_count_by_address(&self, address: [u8; 20]) -> i64 {
        let db = self.db.lock().await;
        get_transaction_count_by_address(&db.pool, address)
            .await
            .unwrap()
    }

    pub async fn get_transaction_count(&self) -> Option<i64> {
        let db = self.db.lock().await;
        get_transaction_count(&db.pool).await.ok()
    }

    pub async fn deposit(&self, address: [u8; 20], value: i64) {
        let db = self.db.lock().await;
        deposit(&db.pool, address, value).await.unwrap()
    }

    pub async fn run_transaction(
        &self,
        signed_transaction: SignedTransaction,
    ) -> anyhow::Result<[u8; 32]> {
        let db = self.db.lock().await;
        let mut transaction = crate::db::Transaction::new(&db.pool.clone(), &signed_transaction)
            .await
            .map_err(|e| {
                println!("{:?}", e);
                anyhow!("new transacion error")
            })?;
        if signed_transaction.to() == SYSTEM_ADDRESS {
            self.run_system_transaction(&mut transaction, &signed_transaction)
                .await?
        } else if signed_transaction.is_transfer() {
            transaction
                .transfer(
                    signed_transaction.signer(),
                    signed_transaction.to(),
                    signed_transaction.value(),
                )
                .await
                .unwrap();
        };
        transaction.commit().await.map_err(|e| {
            println!("{:?}", e);
            anyhow!("new transacion error")
        })?;
        Ok(signed_transaction.hash())
    }

    pub async fn run_system_transaction<'a>(
        &self,
        transaction: &mut crate::db::Transaction<'a>,
        signed_transaction: &SignedTransaction,
    ) -> anyhow::Result<()> {
        match signed_transaction.transaction.input.get(0..4) {
            Some(selector) if selector == UPGRADE_BY_MESSAGE => {
                let (upgrade_by_message, signature, verifying_key) =
                    UpgradeByMessage::decode(&signed_transaction.transaction.input[4..]).await?;
                let amount = upgrade_by_message
                    .validate(
                        &[signature.to_vec(), verifying_key.to_sec1_bytes().to_vec()].concat(),
                    )
                    .await?;
                // upgrade_by_message
                //     .execute(pool, signed_transaction, amount)
                //     .await?;
                transaction
                    .upgrade(
                        1,
                        signed_transaction.hash(),
                        upgrade_by_message.inputs,
                        signed_transaction.signer(),
                        amount,
                    )
                    .await
                    .map_err(|e| crate::error::Error::Error(e.to_string()))?
            }
            _ => return Err(anyhow!("invalid transaction")),
        };

        Ok(())
    }
}

//10000000000i64);
// 10 ^ 18 (ETH) / 10 ^ 8 (BTC)
// const SCALING_FACTOR: U256 = U256::from_be_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 84, 11, 228, 0]);
const SCALING_FACTOR: i64 = i64::pow(10, 10);

pub fn scale_down(n: U256) -> i64 {
    println!("x:{}", SCALING_FACTOR);
    i64::from_le_bytes(
        (n / U256::from(SCALING_FACTOR)).to_le_bytes_vec()[0..8]
            .try_into()
            .unwrap_or(Default::default()),
    )
}

pub fn scale_up(n: i64) -> U256 {
    U256::from(n) * U256::from(SCALING_FACTOR)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    use revm::{
        StateBuilder,
    };
    use sqlx::{PgPool};
    
    
    

    #[sqlx::test]
    async fn transfer(pool: PgPool) -> sqlx::Result<()> {
        // const ALICE: [u8; 20] = [1u8; 20];
        // const BOB: [u8; 20] = [2u8; 20];

        // let empty_db = EmptyDB::new();
        let pg_db = postgres::PgDb::new(pool.clone());
        // let cache_db = CacheDB::new(p_db.clone());
        let _state = StateBuilder::new_with_database(pg_db.clone()).build();
        // let mut evm = Evm::builder()
        //     .with_spec_id(SpecId::CANCUN)
        //     .with_db(&mut state)
        //     .build();
        // _deploy_test_contract(&mut evm);
        // let mut changes = HashMap::new();
        // let mut account = Account {
        //     info: AccountInfo {
        //         balance: U256::from(9999999),
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // };
        // account.mark_touch();
        // changes.insert(Address::from([0; 20]), account);

        // rocks_db.commit(changes);

        // deposit(&pool, ALICE, 9999999999).await.unwrap();
        // evm = evm
        //     .modify()
        //     .modify_tx_env(|etx| {
        //         etx.value = U256::from(1);
        //         etx.transact_to = TransactTo::Call(Address::new([1; 20]))
        //     })
        //     .build();
        // let x = evm.transact().unwrap();
        // println!("{:?}", x.state);
        // assert_eq!(get_balance(&pool, BOB).await.unwrap(), 1);

        Ok(())
    }
}
