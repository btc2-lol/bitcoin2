use revm::{
    db::{DatabaseCommit, DatabaseRef},
    primitives::{Account, AccountInfo, Address, Bytecode, HashMap, B256, U256},
    Database,
};
use sqlx::PgPool;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PgDb {
    pub pool: PgPool,
}

impl PgDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DatabaseCommit for PgDb {
    fn commit(&mut self, _changes: HashMap<Address, Account>) {
        unimplemented!()
    }
}

impl Database for PgDb {
    type Error = std::io::Error;

    fn basic(&mut self, _address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        unimplemented!()
    }

    fn code_by_hash(&mut self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        unimplemented!()
    }

    fn storage(&mut self, _address: Address, _index: U256) -> Result<U256, Self::Error> {
        unimplemented!();
    }

    fn block_hash(&mut self, _number: U256) -> Result<B256, Self::Error> {
        unimplemented!()
    }

    fn transfer(&mut self, _from: Address, _to: Address, _value: U256) {
        unimplemented!()
    }
}

impl DatabaseRef for PgDb {
    type Error = std::io::Error;

    fn basic_ref(&self, _address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        unimplemented!()
    }

    fn code_by_hash_ref(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        unimplemented!()
    }

    fn storage_ref(&self, _address: Address, _index: U256) -> Result<U256, Self::Error> {
        unimplemented!();
    }

    fn block_hash_ref(&self, _number: U256) -> Result<B256, Self::Error> {
        unimplemented!()
    }
}
