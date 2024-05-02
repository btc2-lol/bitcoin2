use super::signature::Signature;
use crate::evm::{scale_down, scale_up, U256};
use borsh::{BorshDeserialize, BorshSerialize};
use k256::{ecdsa, ecdsa::RecoveryId};
use reth_primitives::{Address, TransactionKind, TransactionSigned, TxType};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid to field length")]
    InvalidToFieldLength,
    #[error("only legacy ethereum transactions supported")]
    OnlyLegacyTransactions,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Transaction {
    pub chain_id: Option<i64>,
    pub nonce: i64,
    pub gas_price: u128,
    pub gas_limit: i64,
    pub to: Option<[u8; 20]>,
    pub value: i64,
    pub input: Vec<u8>,
}

impl From<&reth_primitives::Signature> for Signature {
    fn from(signature: &reth_primitives::Signature) -> Self {
        Self(
            ecdsa::Signature::from_slice(
                &[signature.r.to_be_bytes_vec(), signature.s.to_be_bytes_vec()].concat(),
            )
            .unwrap(),
            RecoveryId::new(signature.odd_y_parity, false),
        )
    }
}
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Signature,
}
impl SignedTransaction {
    pub fn decode(mut bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(TransactionSigned::decode_rlp_legacy_transaction(&mut bytes)
            .unwrap()
            .try_into()?)
    }

    pub fn signer(&self) -> [u8; 20] {
        TransactionSigned::try_from(self)
            .unwrap()
            .recover_signer()
            .unwrap()
            .0
            .try_into()
            .unwrap()
    }

    pub fn is_transfer(&self) -> bool {
        true
    }

    pub fn value(&self) -> i64 {
        scale_down(TransactionSigned::try_from(self).unwrap().value())
    }
    pub fn to(&self) -> [u8; 20] {
        TransactionSigned::try_from(self)
            .unwrap()
            .transaction
            .to()
            .unwrap()
            .0
            .try_into()
            .unwrap()
    }

    pub fn hash(&self) -> [u8; 32] {
        TransactionSigned::try_from(self)
            .unwrap()
            .signature_hash()
            .0
            .try_into()
            .unwrap()
    }
}

impl TryFrom<&SignedTransaction> for reth_primitives::TransactionSigned {
    type Error = Box<dyn std::error::Error>;
    fn try_from(signed_transaction: &SignedTransaction) -> std::result::Result<Self, Self::Error> {
        let SignedTransaction {
            transaction,
            signature,
        } = signed_transaction;
        let binding = signature.0.to_bytes().to_vec();
        let (r, s) = binding.split_at(32);
        let to = if let Some(to) = transaction.to {
            TransactionKind::Call(Address::new(to))
        } else {
            TransactionKind::Create
        };

        Ok(TransactionSigned {
            transaction: reth_primitives::Transaction::Legacy(reth_primitives::TxLegacy {
                chain_id: transaction
                    .chain_id
                    .map(|chain_id| chain_id.try_into())
                    .transpose()?,
                gas_limit: transaction.gas_limit.try_into()?,
                gas_price: transaction.gas_price,
                to,
                value: scale_up(transaction.value),
                input: transaction.input.clone().into(),
                nonce: transaction.nonce.try_into()?,
            }),
            signature: reth_primitives::Signature {
                r: U256::from_be_bytes(<&[u8] as TryInto<[u8; 32]>>::try_into(r)?),
                s: U256::from_be_bytes(<&[u8] as TryInto<[u8; 32]>>::try_into(s)?),
                odd_y_parity: signature.1.is_y_odd(),
            },
            hash: Default::default(),
        })
    }
}

impl TryFrom<reth_primitives::TransactionSigned> for SignedTransaction {
    type Error = Box<dyn std::error::Error>;

    fn try_from(transaction: reth_primitives::TransactionSigned) -> Result<Self, Self::Error> {
        if transaction.tx_type() != TxType::Legacy {
            return Err(Box::new(Error::OnlyLegacyTransactions));
        }
        Ok(SignedTransaction {
            transaction: Transaction {
                nonce: transaction.nonce().try_into()?,
                chain_id: transaction
                    .chain_id()
                    .map(|chain_id| chain_id.try_into())
                    .transpose()?,
                gas_limit: transaction.gas_limit().try_into()?,
                gas_price: transaction.max_fee_per_gas(),
                to: transaction
                    .to()
                    .map(|to| {
                        to.0.to_vec()
                            .try_into()
                            .map_err(|_| Error::InvalidToFieldLength)
                    })
                    .transpose()?,
                value: scale_down(transaction.value()),
                input: transaction.input().to_vec(),
            },
            signature: transaction.signature().into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SignedTransaction;
    use hex_lit::hex;

    #[test]
    fn test() {
        let transaction = SignedTransaction::decode(&hex!("f8690180825208943073ac44aa1b95f2fe71bb2eb36b9ce27892f8ee8806f05b59d3b20000808201b9a0d95066012c1af3689ac24030b965a81211b506022d4db117bf90b4a22ccaf981a03c818c75f0634ee921cbcb290371c5e14e76768db4f18900753dbcce651978eb").to_vec()).unwrap();
        let btc2_tx: SignedTransaction =
            borsh::from_slice(&borsh::to_vec(&transaction).unwrap()).unwrap();
        assert_eq!(
            hex::encode(btc2_tx.signer()),
            "f204ee5596cabc6ec60e5e92fd412ea7f856b625"
        );
    }
}
