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

impl digest::Update for Hasher {
    #[inline]
    fn update(&mut self, data: impl AsRef<[u8]>) {
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
    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>) {
        out.copy_from_slice(self.finalize().as_bytes());
    }

    #[inline]
    fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        out.copy_from_slice(self.finalize().as_bytes());
        self.reset();
    }
}

impl digest::ExtendableOutput for Hasher {
    type Reader = OutputReader;

    #[inline]
    fn finalize_xof(self) -> Self::Reader {
        Hasher::finalize_xof(&self)
    }

    #[inline]
    fn finalize_xof_reset(&mut self) -> Self::Reader {
        let reader = Hasher::finalize_xof(self);
        self.reset();
        reader
    }
}

impl digest::XofReader for OutputReader {
    #[inline]
    fn read(&mut self, buffer: &mut [u8]) {
        self.fill(buffer);
    }
}

impl crypto_mac::NewMac for Hasher {
    type KeySize = U32;

    #[inline]
    fn new(key: &crypto_mac::Key<Self>) -> Self {
        let key_bytes: [u8; 32] = (*key).into();
        Hasher::new_keyed(&key_bytes)
    }
}

impl crypto_mac::Mac for Hasher {
    type OutputSize = U32;

    #[inline]
    fn update(&mut self, data: &[u8]) {
        self.update(data);
    }

    #[inline]
    fn reset(&mut self) {
        self.reset();
    }

    #[inline]
    fn finalize(self) -> crypto_mac::Output<Self> {
        crypto_mac::Output::new(digest::Digest::finalize(self))
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        digest::Digest::update(&mut hasher2, b"xxx");
        digest::Digest::reset(&mut hasher2);
        digest::Digest::update(&mut hasher2, b"foo");
        digest::Digest::update(&mut hasher2, b"bar");
        digest::Digest::update(&mut hasher2, b"baz");
        let out2 = digest::Digest::finalize(hasher2.clone());
        let mut xof2 = [0; 301];
        digest::XofReader::read(
            &mut digest::ExtendableOutput::finalize_xof(hasher2.clone()),
            &mut xof2,
        );
        assert_eq!(out1.as_bytes(), &out2[..]);
        assert_eq!(xof1[..], xof2[..]);

        // Again with the resetting variants.
        let mut hasher3: crate::Hasher = digest::Digest::new();
        digest::Digest::update(&mut hasher3, b"foobarbaz");
        let mut out3 = [0; 32];
        digest::FixedOutput::finalize_into_reset(
            &mut hasher3,
            GenericArray::from_mut_slice(&mut out3),
        );
        digest::Digest::update(&mut hasher3, b"foobarbaz");
        let mut out4 = [0; 32];
        digest::FixedOutput::finalize_into_reset(
            &mut hasher3,
            GenericArray::from_mut_slice(&mut out4),
        );
        digest::Digest::update(&mut hasher3, b"foobarbaz");
        let mut xof3 = [0; 301];
        digest::XofReader::read(
            &mut digest::ExtendableOutput::finalize_xof_reset(&mut hasher3),
            &mut xof3,
        );
        digest::Digest::update(&mut hasher3, b"foobarbaz");
        let mut xof4 = [0; 301];
        digest::XofReader::read(
            &mut digest::ExtendableOutput::finalize_xof_reset(&mut hasher3),
            &mut xof4,
        );
        assert_eq!(out1.as_bytes(), &out3[..]);
        assert_eq!(out1.as_bytes(), &out4[..]);
        assert_eq!(xof1[..], xof3[..]);
        assert_eq!(xof1[..], xof4[..]);
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
        let mut hasher2: crate::Hasher = crypto_mac::NewMac::new(&generic_key);
        crypto_mac::Mac::update(&mut hasher2, b"xxx");
        crypto_mac::Mac::reset(&mut hasher2);
        crypto_mac::Mac::update(&mut hasher2, b"foo");
        crypto_mac::Mac::update(&mut hasher2, b"bar");
        crypto_mac::Mac::update(&mut hasher2, b"baz");
        let out2 = crypto_mac::Mac::finalize(hasher2);
        assert_eq!(out1.as_bytes(), out2.into_bytes().as_slice());
    }
}
