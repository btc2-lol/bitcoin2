mod rocksdb;
#[cfg(test)]
mod test_utils;
pub mod transaction;
use reth_primitives::U256;
pub use transaction::SignedTransaction;

// 10 ^ 14
const SCALING_FACTOR: U256 = U256::from_be_bytes([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 90, 243, 16, 122,
    64, 0,
]);

pub fn scale_down(n: U256) -> i64 {
    i64::from_le_bytes(
        (n / SCALING_FACTOR).to_le_bytes_vec()[0..8]
            .try_into()
            .unwrap_or(Default::default()),
    )
}

pub fn scale_up(n: i64) -> U256 {
    U256::from(n) * SCALING_FACTOR
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::rocksdb::DB;
    use revm::{
        db::{CacheDB, EmptyDB},
        primitives::{Account, AccountInfo, Address, SpecId, U256},
        DatabaseCommit, Evm, StateBuilder,
    };
    use std::collections::HashMap;
    use tempfile::tempdir;
    use test_utils::_deploy_test_contract;

    #[test]
    fn evm() {
        let dir = tempdir().unwrap();
        let db = DB::open_default(dir.path().join("test.rocksdb")).unwrap();
        let empty_db = EmptyDB::new();
        let mut rocks_db = rocksdb::RocksDb::_new(db, empty_db);
        let cache_db = CacheDB::new(rocks_db.clone());
        let mut state = StateBuilder::new_with_database(cache_db.clone()).build();
        let mut evm = Evm::builder()
            .with_spec_id(SpecId::CANCUN)
            .with_db(&mut state)
            .build();
        _deploy_test_contract(&mut evm);
        let mut changes = HashMap::new();
        let mut account = Account {
            info: AccountInfo {
                balance: U256::from(9999999),
                ..Default::default()
            },
            ..Default::default()
        };
        account.mark_touch();
        changes.insert(Address::from([0; 20]), account);

        rocks_db.commit(changes);
        evm = evm
            .modify()
            .modify_tx_env(|etx| {
                etx.value = U256::from(1);
            })
            .build();
        evm.transact().unwrap();
    }
}
