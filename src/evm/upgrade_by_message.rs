use crate::{bitcoin_legacy, bitcoin_legacy::utxos, error::{Result}};
use ethers_core::abi::ParamType;
use k256::ecdsa::VerifyingKey;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UpgradeByMessage {
    #[serde(alias = "Action")]
    action: String,
    #[serde(alias = "Destination Chain ID")]
    destination_chain_id: i64,
    #[serde(alias = "Destination Address", deserialize_with = "from_0x_prefixed_hex")]
    destination_address: [u8; 20],
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

fn from_0x_prefixed_hex<'de, D>(deserializer: D) -> std::result::Result<[u8; 20], D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    if s[0..2] != *"0x" {
        return Err(D::Error::custom("expected hex value to start with 0x"));
    }
    Ok(hex::decode(&s[2..])
        .map_err(D::Error::custom)?
        .try_into()
        .map_err(|_| D::Error::custom("Invalid Length"))?)
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
    pub async fn validate(&self, unlocking_script: &[u8], destination_address: [u8; 20]) -> Result<i64> {
        if self.destination_address != destination_address {
            return Err(crate::error::Error::Error("Invalid destination address".to_string()));
        };

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
}
