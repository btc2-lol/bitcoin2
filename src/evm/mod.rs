pub mod postgres;
pub mod upgrade_by_message;

use crate::{
    constants::{SYSTEM_ADDRESS, UPGRADE_BY_MESSAGE},
    db::{
        deposit, get_balance, get_transaction_count, get_transaction_count_by_address, Transaction,
    },
    error::{Error, Result},
};
use reth_primitives::U256;
pub use reth_primitives::{transaction::TransactionSigned, Address};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use upgrade_by_message::UpgradeByMessage;

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

    // pub async fn get_transaction(
    //     &self,
    //     block_number: Option<i64>,
    //     hash: Option<[u8; 32]>,
    // ) -> Result<TransactionSigned> {
    //     let db = self.db.lock().await;
    //     Ok(get_transaction(&db.pool, block_number, hash).await?.1)
    // }

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

    pub async fn run_transaction(&self, signed_transaction: &TransactionSigned) -> Result<i64> {
        let db = self.db.lock().await;

        let mut transaction = Transaction::new(&db.pool.clone(), &signed_transaction).await?;
        if signed_transaction.to() == Some(Address::from(SYSTEM_ADDRESS)) {
            self.run_system_transaction(&mut transaction, &signed_transaction)
                .await?
        } else {
            transaction
                .transfer(
                    signed_transaction
                        .recover_signer()
                        .ok_or(Error::InvalidSignature)
                        .map(|signer| signer.to_vec().try_into().unwrap())?,
                    signed_transaction
                        .to()
                        .ok_or(Error::InvalidTransaction)?
                        .to_vec()
                        .try_into()
                        .unwrap(),
                    scale_down(signed_transaction.value()),
                )
                .await?;
        };
        let transaction_id = transaction.commit().await?;

        Ok(transaction_id)
    }

    pub async fn run_system_transaction<'a>(
        &self,
        transaction: &mut crate::db::Transaction<'a>,
        signed_transaction: &TransactionSigned,
    ) -> Result<()> {
        match signed_transaction.transaction.input().get(0..4) {
            Some(selector) if selector == UPGRADE_BY_MESSAGE => {
                let (upgrade_by_message, signature, verifying_key) =
                    UpgradeByMessage::decode(&signed_transaction.transaction.input()[4..]).await?;
                let signer = signed_transaction
                .recover_signer()
                .ok_or(Error::InvalidSignature)?
                .to_vec()
                .try_into()?;
                let amount = upgrade_by_message
                    .validate(
                        &[signature.to_vec(), verifying_key.to_sec1_bytes().to_vec()].concat(),
                        signer
                    )
                    .await?;
                let _ = transaction
                    .upgrade(
                        upgrade_by_message.inputs,
                        signer,
                        amount,
                    )
                    .await?;
                println!("{} unlocked {} BTC2", hex::encode(signer), amount);
                Ok::<(), Error>(())
            }
            _ => return Err(Error::FunctionNotFound),
        }
    }
}

// 10 ^ 18 (ETH) / 10 ^ 8 (BTC) = 10 ^ 10
pub const SCALING_FACTOR: i64 = i64::pow(10, 10);

pub fn scale_down(n: U256) -> i64 {
    i64::from_le_bytes(
        (n / U256::from(SCALING_FACTOR)).to_le_bytes_vec()[0..8]
            .try_into()
            .unwrap_or(Default::default()),
    )
}

pub fn scale_up(n: i64) -> U256 {
    U256::from(n) * U256::from(SCALING_FACTOR)
}
