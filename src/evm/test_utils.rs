// use alloy_json_abi::JsonAbi;
// use k256::{
//     ecdsa::{signature::Signer, Signature, SigningKey},
//     SecretKey,
// };
// use rand_core::OsRng;
// use reth_primitives::TransactionSigned;
// use reth_primitives::U256;
// use crate::evm::scale_up;
// use crate::evm::transaction::scale_down;
// use ethereum_tx_sign::AccessListTransaction;
// use ethereum_tx_sign::Transaction;
use revm::{Database, Evm};
// use reth_primitives::transaction::TxLegacy;
// use hex_lit::hex;
// use reth_primitives::{TransactionSigned, TxLegacy};

pub fn _deploy_test_contract<EXT, DB: Database>(_evm: &mut Evm<EXT, DB>) {
    // println!("{:?}", get_test_contract_bytecode());
    // let mut x = hex!("02f8720119830f424085020f7cc6a082520894cb19327b0f006c4f481b22951eaa11ce0ddcc2f7880a69474944e7c96080c080a0c1b4fa65452c500ee173cacd81a1011d33fea8b0ccea5aeaad799b7673deae09a025f6a0def5e1e04eb58bd2dcc67c2357eec0df49386bc4fee1d51dbf7e040f89");
    // println!("{:?}", TransactionSigned::decode_enveloped_typed_transaction(&mut &x[..]));
    // let contract: serde_json::Value = serde_json::from_str(&json).unwrap();
    // println!("{:?}", contract.get::<&str>("bytecode").unwrap()[2..]);
    // for item in abi.items() {
    //     println!("{item:?}");
    // }
}
fn _sign_tx() {
    // let signing_key = SigningKey::random(&mut OsRng);

    // let tx = AccessListTransaction {
    //     chain: 1,
    //     nonce: 0,
    //     gas_price: 0,
    //     gas: 0,
    //     to: Some([0; 20]),
    //     value: u128::from_le_bytes(
    //         scale_up(1000).to_le_bytes_vec()[0..8]
    //             .try_into()
    //             .unwrap_or(Default::default()),
    //     ),
    //     data: vec![],
    //     access_list: Default::default(),
    // };
    // let ecdsa = tx.ecdsa(&signing_key.to_bytes()).unwrap();
    // let mut transaction_bytes = tx.sign(&ecdsa);
    // let mut n: U256 = "100000000000000".parse().unwrap();
    // const WEI_PER_ETH: U256 = U256::from_be_bytes([
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 224, 182,
    //     179, 167, 100, 0, 0,
    // ]);
    // const TEN_THOUSAND_U256: U256 = U256::from_be_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 39, 16]);
    // const DECIMALS: usize = 4;
    // const SCALING_CONST: U256 = WEI_PER_ETH / TEN_THOUSAND_U256;
    // const TEN_THOUSAND_U256: U256 = U256::from(10000);

    // let mut bytes = n.as_le_bytes().clone().to_vec();
    // bytes.reverse();
    // println!("{:?}", scale_down(scale_up(10)));

    // println!("{:?}", TransactionSigned::decode_enveloped_typed_transaction(&mut &transaction_bytes[..]));
}
fn _get_test_contract_bytecode() -> Vec<u8> {
    let path = "contracts/artifacts/contracts/TestContract.sol/TestContract.json";
    let json = std::fs::read_to_string(path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let hex_str = v["bytecode"].as_str().unwrap();
    let trimmed_hex = hex_str.trim_start_matches("0x");
    hex::decode(trimmed_hex).unwrap()
}
