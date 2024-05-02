use crate::bitcoin_legacy;
use ethers_core::abi::ParamType;

use crate::error::Result;
use k256::ecdsa::VerifyingKey;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::bitcoin_legacy::utxos;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UpgradeByMessage {
    #[serde(alias = "Action")]
    action: String,
    #[serde(alias = "Destination Chain ID")]
    destination_chain_id: i64,
    #[serde(alias = "Inputs")]
    pub inputs: Vec<Outpoint>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Outpoint {
    #[serde(alias = "Hash", deserialize_with = "from_hex")]
    pub hash: [u8; 32],
    #[serde(alias = "Index")]
    pub index: i16,
}

fn from_hex<'de, D>(deserializer: D) -> std::result::Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(hex::decode(s)
        .map_err(D::Error::custom)?
        .try_into()
        .map_err(|_| D::Error::custom(""))?)
}

impl UpgradeByMessage {
    pub async fn decode(arguments: &[u8]) -> Result<(Self, [u8; 65], VerifyingKey)> {
        if let [message, signature] =
            ethers_core::abi::decode(&[ParamType::String, ParamType::Bytes], &arguments)
                .unwrap()
                .as_slice()
        {
            let signature = signature.clone().into_bytes().unwrap().try_into().unwrap();
            let verifying_key = bitcoin_legacy::recover_from_msg(
                message.clone().into_string().unwrap().as_bytes().to_vec(),
                signature,
            )
            .unwrap();
            Ok((
                serde_yaml::from_str(&message.to_string()).unwrap(),
                signature,
                verifying_key,
            ))

            // upgrade_message.validate(
            //     &[
            //         signature.to_vec(), verifying_key.to_sec1_bytes().to_vec()
            //     ].concat()
            // )
        } else {
            Err(crate::error::Error::Error("parse error".into()))
        }
    }
    pub async fn validate(&self, unlocking_script: &[u8]) -> Result<i64> {
        self.inputs
            .iter()
            .map(|input| {
                utxos::validate(
                    &utxos::Outpoint {
                        hash: input
                            .hash
                            .iter()
                            .rev()
                            .cloned()
                            .collect::<Vec<u8>>()
                            .try_into()
                            .unwrap(),
                        index: input.index as u16,
                    },
                    unlocking_script,
                )
            })
            .sum::<Result<i64>>()
            .map_err(|e| crate::error::Error::Error(e.to_string()))
    }

    // pub async fn execute(
    //     self,
    //     pool: Pool<Postgres>,
    //     signed_transaction: &SignedTransaction,
    //     amount: i64,
    // ) -> Result<()> {
    //     upgrade(
    //         &pool,
    //         1,
    //         signed_transaction.hash(),
    //         self.inputs,
    //         signed_transaction.signer(),
    //         amount,
    //     )
    //     .await
    //     .map_err(|e| crate::error::Error::Error(e.to_string()))
    // }
}
