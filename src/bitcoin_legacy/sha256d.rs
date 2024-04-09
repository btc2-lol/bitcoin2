use digest::{
    generic_array::{typenum::U32, GenericArray},
    FixedOutput, HashMarker, OutputSizeUser, Update,
};
use sha2::{Digest, Sha256};

#[derive(Default, Clone)]
pub struct Sha256d {
    hasher: Sha256,
}

impl OutputSizeUser for Sha256d {
    type OutputSize = U32;
}

impl Update for Sha256d {
    fn update(&mut self, data: &[u8]) {
        Update::update(&mut self.hasher, data)
    }
}

impl FixedOutput for Sha256d {
    fn finalize_into(self, data: &mut GenericArray<u8, <Self as OutputSizeUser>::OutputSize>) {
        FixedOutput::finalize_into(self.hasher, data);
        let mut second = Sha256::new();
        Update::update(&mut second, data);
        FixedOutput::finalize_into(second, data);
    }
}

impl HashMarker for Sha256d {}
