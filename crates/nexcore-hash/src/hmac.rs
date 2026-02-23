//! HMAC-SHA256 implementation per RFC 2104.
//!
//! Zero external dependencies. Uses [`super::sha256::Sha256`] internally.

use crate::sha256::{Sha256, Sha256Digest};

/// HMAC-SHA256 key + state.
///
/// # Examples
/// ```
/// use nexcore_hash::hmac::HmacSha256;
///
/// let mac = HmacSha256::mac(b"my secret key", b"hello world");
/// assert_eq!(mac.len(), 32);
/// ```
pub struct HmacSha256 {
    outer_key_pad: [u8; 64],
    inner_hasher: Sha256,
}

const BLOCK_SIZE: usize = 64; // SHA-256 block size in bytes

impl HmacSha256 {
    /// Create a new HMAC-SHA256 instance with the given key.
    ///
    /// Keys longer than 64 bytes are hashed first (per RFC 2104 §2).
    pub fn new(key: &[u8]) -> Self {
        // Step 1: normalize key to block size
        let mut key_block = [0u8; BLOCK_SIZE];
        if key.len() > BLOCK_SIZE {
            let hashed = Sha256::digest(key);
            key_block[..32].copy_from_slice(&hashed);
        } else {
            key_block[..key.len()].copy_from_slice(key);
        }

        // Step 2: compute inner and outer padded keys
        let mut inner_key_pad = [0x36u8; BLOCK_SIZE];
        let mut outer_key_pad = [0x5cu8; BLOCK_SIZE];
        for i in 0..BLOCK_SIZE {
            inner_key_pad[i] ^= key_block[i];
            outer_key_pad[i] ^= key_block[i];
        }

        // Step 3: start inner hash with ipad
        let mut inner_hasher = Sha256::new();
        inner_hasher.update(&inner_key_pad);

        Self {
            outer_key_pad,
            inner_hasher,
        }
    }

    /// Create from a byte slice. Returns error if key is empty.
    pub fn new_from_slice(key: &[u8]) -> Result<Self, HmacError> {
        if key.is_empty() {
            return Err(HmacError::InvalidKeyLength);
        }
        Ok(Self::new(key))
    }

    /// Feed data into the HMAC computation.
    pub fn update(&mut self, data: &[u8]) {
        self.inner_hasher.update(data);
    }

    /// Finalize and return the MAC tag.
    pub fn finalize(self) -> Sha256Digest {
        // Complete inner hash
        let inner_hash = self.inner_hasher.finalize();

        // Outer hash: H(opad || inner_hash)
        let mut outer = Sha256::new();
        outer.update(&self.outer_key_pad);
        outer.update(&inner_hash);
        outer.finalize()
    }

    /// One-shot HMAC computation.
    pub fn mac(key: &[u8], data: &[u8]) -> Sha256Digest {
        let mut hmac = Self::new(key);
        hmac.update(data);
        hmac.finalize()
    }

    /// Verify a MAC tag in constant time.
    pub fn verify(key: &[u8], data: &[u8], expected: &[u8]) -> bool {
        let computed = Self::mac(key, data);
        constant_time_eq(&computed, expected)
    }
}

/// Error type for HMAC operations.
#[derive(Debug, Clone)]
pub enum HmacError {
    /// Key length is zero.
    InvalidKeyLength,
}

impl std::fmt::Display for HmacError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKeyLength => write!(f, "HMAC key must not be empty"),
        }
    }
}

impl std::error::Error for HmacError {}

/// Constant-time byte comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    // RFC 4231 Test Case 1
    #[test]
    fn rfc4231_test_case_1() {
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let mac = HmacSha256::mac(&key, data);
        assert_eq!(
            hex(&mac),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    // RFC 4231 Test Case 2
    #[test]
    fn rfc4231_test_case_2() {
        let key = b"Jefe";
        let data = b"what do ya want for nothing?";
        let mac = HmacSha256::mac(key, data);
        assert_eq!(
            hex(&mac),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    // RFC 4231 Test Case 3
    #[test]
    fn rfc4231_test_case_3() {
        let key = [0xaau8; 20];
        let data = [0xddu8; 50];
        let mac = HmacSha256::mac(&key, &data);
        assert_eq!(
            hex(&mac),
            "773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe"
        );
    }

    // RFC 4231 Test Case 6 (key > block size)
    #[test]
    fn rfc4231_test_case_6() {
        let key = [0xaau8; 131];
        let data = b"Test Using Larger Than Block-Size Key - Hash Key First";
        let mac = HmacSha256::mac(&key, data);
        assert_eq!(
            hex(&mac),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn verify_correct() {
        let key = b"test-key";
        let data = b"test-data";
        let mac = HmacSha256::mac(key, data);
        assert!(HmacSha256::verify(key, data, &mac));
    }

    #[test]
    fn verify_wrong_data() {
        let key = b"test-key";
        let mac = HmacSha256::mac(key, b"correct");
        assert!(!HmacSha256::verify(key, b"wrong", &mac));
    }

    #[test]
    fn verify_wrong_key() {
        let data = b"test-data";
        let mac = HmacSha256::mac(b"correct-key", data);
        assert!(!HmacSha256::verify(b"wrong-key", data, &mac));
    }

    #[test]
    fn incremental_same_as_oneshot() {
        let key = b"my-key";
        let data = b"hello world this is a longer message for testing";
        let oneshot = HmacSha256::mac(key, data);

        let mut hmac = HmacSha256::new(key);
        hmac.update(&data[..11]);
        hmac.update(&data[11..]);
        let incremental = hmac.finalize();

        assert_eq!(oneshot, incremental);
    }

    #[test]
    fn empty_key_error() {
        assert!(HmacSha256::new_from_slice(b"").is_err());
    }
}
