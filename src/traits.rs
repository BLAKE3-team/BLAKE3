//! Implementations of commonly used traits like
//! [`digest::Digest`](https://crates.io/crates/digest) and
//! [`crypto_mac::Mac`](https://crates.io/crates/crypto-mac).

pub use crypto_mac;
pub use digest;

use crate::{Hasher, OutputReader};
use digest::generic_array::{
    typenum::{U32, U64},
    GenericArray,
};

impl digest::BlockInput for Hasher {
    type BlockSize = U64;
}

impl digest::Input for Hasher {
    #[inline]
    fn input<B: AsRef<[u8]>>(&mut self, data: B) {
        self.update(data.as_ref());
    }
}

impl digest::Reset for Hasher {
    #[inline]
    fn reset(&mut self) {
        self.reset(); // the inherent method
    }
}

impl digest::FixedOutput for Hasher {
    type OutputSize = U32;

    #[inline]
    fn fixed_result(self) -> GenericArray<u8, Self::OutputSize> {
        GenericArray::clone_from_slice(self.finalize().as_bytes())
    }
}

impl digest::ExtendableOutput for Hasher {
    type Reader = OutputReader;

    #[inline]
    fn xof_result(self) -> Self::Reader {
        self.finalize_xof()
    }
}

impl digest::XofReader for OutputReader {
    #[inline]
    fn read(&mut self, buffer: &mut [u8]) {
        self.fill(buffer);
    }
}

impl crypto_mac::Mac for Hasher {
    type OutputSize = U32;
    type KeySize = U32;

    #[inline]
    fn new(key: &GenericArray<u8, Self::KeySize>) -> Self {
        let key_bytes: [u8; 32] = (*key).into();
        Hasher::new_keyed(&key_bytes)
    }

    #[inline]
    fn input(&mut self, data: &[u8]) {
        self.update(data);
    }

    #[inline]
    fn reset(&mut self) {
        self.reset();
    }

    #[inline]
    fn result(self) -> crypto_mac::MacResult<Self::OutputSize> {
        crypto_mac::MacResult::new((*self.finalize().as_bytes()).into())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_digest_traits() {
        // Inherent methods.
        let mut hasher1 = crate::Hasher::new();
        hasher1.update(b"foo");
        hasher1.update(b"bar");
        hasher1.update(b"baz");
        let out1 = hasher1.finalize();
        let mut xof1 = [0; 301];
        hasher1.finalize_xof().fill(&mut xof1);
        assert_eq!(out1.as_bytes(), &xof1[..32]);

        // Trait implementations.
        let mut hasher2: crate::Hasher = digest::Digest::new();
        digest::Digest::input(&mut hasher2, b"xxx");
        digest::Digest::reset(&mut hasher2);
        digest::Digest::input(&mut hasher2, b"foo");
        digest::Digest::input(&mut hasher2, b"bar");
        digest::Digest::input(&mut hasher2, b"baz");
        let out2 = digest::Digest::result(hasher2.clone());
        let mut xof2 = [0; 301];
        digest::XofReader::read(
            &mut digest::ExtendableOutput::xof_result(hasher2),
            &mut xof2,
        );
        assert_eq!(out1.as_bytes(), &out2[..]);
        assert_eq!(xof1[..], xof2[..]);
    }

    #[test]
    fn test_mac_trait() {
        // Inherent methods.
        let key = b"some super secret key bytes fooo";
        let mut hasher1 = crate::Hasher::new_keyed(key);
        hasher1.update(b"foo");
        hasher1.update(b"bar");
        hasher1.update(b"baz");
        let out1 = hasher1.finalize();

        // Trait implementation.
        let generic_key = (*key).into();
        let mut hasher2: crate::Hasher = crypto_mac::Mac::new(&generic_key);
        crypto_mac::Mac::input(&mut hasher2, b"xxx");
        crypto_mac::Mac::reset(&mut hasher2);
        crypto_mac::Mac::input(&mut hasher2, b"foo");
        crypto_mac::Mac::input(&mut hasher2, b"bar");
        crypto_mac::Mac::input(&mut hasher2, b"baz");
        let out2 = crypto_mac::Mac::result(hasher2);
        assert_eq!(out1.as_bytes(), out2.code().as_slice());
    }
}
