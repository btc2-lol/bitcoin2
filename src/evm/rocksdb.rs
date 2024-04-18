use revm::{
    db::{DatabaseCommit, DatabaseRef},
    primitives::{
        hash_map::Entry, Account, AccountInfo, Address, Bytecode, HashMap, Log, B256, KECCAK_EMPTY,
        U256,
    },
    Database,
};
use std::{sync::Arc, vec::Vec};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RocksDb<ExtDB> {
    pub accounts: HashMap<Address, DbAccount>,
    pub contracts: HashMap<B256, Bytecode>,
    pub logs: Vec<Log>,
    pub block_hashes: HashMap<U256, B256>,
    pub read_only_db: ExtDB,
    pub db: Arc<rocksdb::DB>,
}

impl<ExtDB> RocksDb<ExtDB> {
    pub fn _new(db: rocksdb::DB, read_only_db: ExtDB) -> Self {
        let mut contracts = HashMap::new();
        contracts.insert(KECCAK_EMPTY, Bytecode::default());
        contracts.insert(B256::ZERO, Bytecode::default());
        Self {
            accounts: HashMap::new(),
            contracts,
            logs: Vec::default(),
            block_hashes: HashMap::new(),
            db: db.into(),
            read_only_db,
        }
    }
}

impl<ExtDB> DatabaseCommit for RocksDb<ExtDB> {
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        for (address, account) in changes {
            if !account.is_touched() {
                continue;
            }
            if account.is_selfdestructed() {
                let db_account = self.accounts.entry(address).or_default();
                db_account.storage.clear();
                db_account.account_state = AccountState::NotExisting;
                // db_account.info = AccountInfo::default();
                continue;
            }
            let _is_newly_created = account.is_created();
            // self.insert_contract(&mut account.info);

            // let db_account = self.accounts.entry(address).or_default();
            let mut db_account: DbAccount = if let Ok(Some(account_bytes)) = self.db.get(address) {
                bincode::decode_from_slice(&account_bytes, bincode::config::standard())
                    .unwrap()
                    .0
            } else {
                Default::default()
            };
            db_account.balance = account.info.balance.to_be_bytes();

            // db_account.account_state = if is_newly_created {
            //     db_account.storage.clear();
            //     AccountState::StorageCleared
            // } else if db_account.account_state.is_storage_cleared() {
            //     // Preserve old account state if it already exists
            //     AccountState::StorageCleared
            // } else {
            //     AccountState::Touched
            // };
            db_account.storage.extend(
                account
                    .storage
                    .into_iter()
                    .map(|(key, value)| (key.to_be_bytes(), value.present_value().to_be_bytes())),
            );

            self.db
                .put(
                    address,
                    bincode::encode_to_vec(db_account, bincode::config::standard()).unwrap(),
                )
                .unwrap();
        }
    }
}

impl<ExtDB: DatabaseRef> Database for RocksDb<ExtDB> {
    type Error = ExtDB::Error;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let (basic, _): (DbAccount, usize) = bincode::decode_from_slice(
            &self.db.get(address).unwrap().unwrap(),
            bincode::config::standard(),
        )
        .unwrap();
        Ok(basic.info())
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        match self.contracts.entry(code_hash) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                // if you return code bytes when basic fn is called this function is not needed.
                Ok(entry
                    .insert(self.read_only_db.code_by_hash_ref(code_hash)?)
                    .clone())
            }
        }
    }

    fn storage(&mut self, _address: Address, _index: U256) -> Result<U256, Self::Error> {
        unimplemented!();
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        match self.block_hashes.entry(number) {
            Entry::Occupied(entry) => Ok(*entry.get()),
            Entry::Vacant(entry) => {
                let hash = self.read_only_db.block_hash_ref(number)?;
                entry.insert(hash);
                Ok(hash)
            }
        }
    }
}

impl<ExtDB: DatabaseRef> DatabaseRef for RocksDb<ExtDB> {
    type Error = ExtDB::Error;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let db_account: DbAccount = if let Ok(Some(account_bytes)) = self.db.get(address) {
            bincode::decode_from_slice(&account_bytes, bincode::config::standard())
                .unwrap()
                .0
        } else {
            Default::default()
        };
        Ok(db_account.info())
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        match self.contracts.get(&code_hash) {
            Some(entry) => Ok(entry.clone()),
            None => self.read_only_db.code_by_hash_ref(code_hash),
        }
    }

    fn storage_ref(&self, _address: Address, _index: U256) -> Result<U256, Self::Error> {
        unimplemented!();
    }

    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        match self.block_hashes.get(&number) {
            Some(entry) => Ok(*entry),
            None => self.read_only_db.block_hash_ref(number),
        }
    }
}

#[derive(Debug, Clone, Default, bincode::Encode, bincode::Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DbAccount {
    balance: [u8; 32],
    nonce: u64,
    code_hash: [u8; 4],
    code: Vec<u8>,
    pub account_state: AccountState,
    pub storage: HashMap<[u8; 4], [u8; 4]>,
}

impl DbAccount {
    pub fn new_not_existing() -> Self {
        Self {
            account_state: AccountState::NotExisting,
            ..Default::default()
        }
    }

    pub fn info(&self) -> Option<AccountInfo> {
        if matches!(self.account_state, AccountState::NotExisting) {
            None
        } else {
            Some(AccountInfo {
                balance: U256::from_be_bytes(self.balance),
                ..Default::default()
            })
        }
    }
}

impl From<Option<AccountInfo>> for DbAccount {
    fn from(from: Option<AccountInfo>) -> Self {
        from.map(Self::from).unwrap_or_else(Self::new_not_existing)
    }
}

impl From<AccountInfo> for DbAccount {
    fn from(_info: AccountInfo) -> Self {
        Self {
            account_state: AccountState::None,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AccountState {
    NotExisting,
    Touched,
    StorageCleared,
    #[default]
    None,
}
