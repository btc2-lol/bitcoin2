use k256::ecdsa::SigningKey;
use k256::ecdsa::VerifyingKey;
use k256::ecdsa::RecoveryId;
use k256::ecdsa::Signature;

pub fn sign(private_key: SigningKey, message: &[u8]) -> [u8; 65] {
    let (signature, recid) = private_key.sign_recoverable(message).unwrap();
    
    [
        signature.to_bytes().as_slice(),
        &[recid.try_into().unwrap()]
    ].concat().try_into().unwrap()
}

pub fn recover_public_key(message: &[u8], signature_bytes: [u8; 65]) -> [u8; 33] {
    let signature = Signature::try_from(&signature_bytes[..64]).unwrap();
    let recovery_id = RecoveryId::try_from(signature_bytes[64] % 4).unwrap();
    println!("{}", VerifyingKey::recover_from_msg(message, &signature, recovery_id).unwrap().to_sec1_bytes().len());
    VerifyingKey::recover_from_msg(message, &signature, recovery_id).unwrap().to_sec1_bytes().to_vec().try_into().unwrap()
}
