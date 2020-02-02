use crate::{Hasher, OutputReader};
use digest::generic_array::{
    typenum::{U32, U64},
    GenericArray,
};
use digest::{BlockInput, ExtendableOutput, FixedOutput, Input, Reset, XofReader};

impl BlockInput for Hasher {
    type BlockSize = U64;
}

impl Input for Hasher {
    fn input<B: AsRef<[u8]>>(&mut self, data: B) {
        self.update(data.as_ref());
    }
}

impl Reset for Hasher {
    fn reset(&mut self) {
        self.reset(); // the inherent method
    }
}

impl FixedOutput for Hasher {
    type OutputSize = U32;

    fn fixed_result(self) -> GenericArray<u8, Self::OutputSize> {
        GenericArray::clone_from_slice(self.finalize().as_bytes())
    }
}

impl ExtendableOutput for Hasher {
    type Reader = OutputReader;

    fn xof_result(self) -> Self::Reader {
        self.finalize_xof()
    }
}

impl XofReader for OutputReader {
    fn read(&mut self, buffer: &mut [u8]) {
        self.fill(buffer);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_digest_traits() {
        // Inherent methods.
        let mut hasher1 = Hasher::new();
        hasher1.update(b"foo");
        hasher1.update(b"bar");
        hasher1.update(b"baz");
        let out1 = hasher1.finalize();
        let mut xof1 = [0; 301];
        hasher1.finalize_xof().fill(&mut xof1);
        assert_eq!(out1.as_bytes(), &xof1[..32]);

        // Trait implementations.
        let mut hasher2 = Hasher::default();
        hasher2.input(b"xxx");
        Reset::reset(&mut hasher2); // avoid the reset() inherent method
        hasher2.input(b"foo");
        hasher2.input(b"bar");
        hasher2.input(b"baz");
        let out2 = hasher2.clone().fixed_result();
        let mut xof2 = [0; 301];
        hasher2.xof_result().read(&mut xof2);
        assert_eq!(out1.as_bytes(), &out2[..]);
        assert_eq!(xof1[..], xof2[..]);
    }
}
