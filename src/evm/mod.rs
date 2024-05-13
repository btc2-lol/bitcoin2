pub mod postgres;
// pub mod transaction;
pub mod upgrade_by_message;

use crate::db::{
    deposit, get_balance, get_transaction_count, get_transaction_count_by_address,
    get_transaction_hash, get_transactions_by_address,
};
use reth_primitives::U256;
pub use reth_primitives::{transaction::TransactionSigned, Address};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
// pub use transaction::SignedTransaction;
use upgrade_by_message::UpgradeByMessage;

const SYSTEM_ADDRESS: [u8; 20] = [0; 20];
// lazy_static! {
//     static ref  SYSTEM_ADDRESS: Address =  Address::from([0; 20]);
// }

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

    pub async fn get_transactions_by_address(
        &self,
        address: [u8; 20],
    ) -> crate::error::Result<Vec<TransactionSigned>> {
        let db = self.db.lock().await;
        get_transactions_by_address(&db.pool, address).await
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
        signed_transaction: TransactionSigned,
    ) -> crate::error::Result<[u8; 32]> {
        println!("run");
        let db = self.db.lock().await;
        let mut transaction =
            crate::db::Transaction::new(&db.pool.clone(), &signed_transaction).await?;
        if signed_transaction.to() == Some(Address::from(SYSTEM_ADDRESS)) {
            self.run_system_transaction(&mut transaction, &signed_transaction)
                .await?
        } else {
            println!("here here here");
            transaction
                .transfer(
                    signed_transaction
                        .recover_signer()
                        .ok_or(crate::error::Error::Error("Invalid Signer".to_string()))
                        .map(|to| -> [u8; 20] { to.to_vec().try_into().unwrap() })?,
                    signed_transaction
                        .to()
                        .map(|to| -> [u8; 20] { to.to_vec().try_into().unwrap() })
                        .unwrap(),
                    scale_down(signed_transaction.value()),
                )
                .await
                .unwrap();
            println!("done");
        };
        let hash = transaction.id_as_hash();
        transaction.commit().await?;
        Ok(hash)
    }

    pub async fn run_system_transaction<'a>(
        &self,
        transaction: &mut crate::db::Transaction<'a>,
        signed_transaction: &TransactionSigned,
    ) -> crate::error::Result<()> {
        match signed_transaction.transaction.input().get(0..4) {
            Some(selector) if selector == UPGRADE_BY_MESSAGE => {
                let (upgrade_by_message, signature, verifying_key) =
                    UpgradeByMessage::decode(&signed_transaction.transaction.input()[4..]).await?;
                let amount = upgrade_by_message
                    .validate(
                        &[signature.to_vec(), verifying_key.to_sec1_bytes().to_vec()].concat(),
                    )
                    .await?;
                transaction
                    .upgrade(
                        1,
                        signed_transaction.hash().try_into().unwrap(),
                        upgrade_by_message.inputs,
                        signed_transaction
                            .recover_signer()
                            .unwrap()
                            .to_vec()
                            .try_into()
                            .unwrap(),
                        amount,
                    )
                    .await
                    .map_err(|e| crate::error::Error::Error(e.to_string()))?
            }
            _ => return Err(crate::error::Error::Error("Contract not found".to_string())),
        };

        Ok(())
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
