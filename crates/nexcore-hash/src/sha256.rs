//! SHA-256 implementation per FIPS 180-4.
//!
//! Zero external dependencies. Pure safe Rust.
//! Validated against NIST CAVP test vectors.

/// SHA-256 digest output (32 bytes).
pub type Sha256Digest = [u8; 32];

/// SHA-256 hasher.
///
/// # Examples
/// ```
/// use nexcore_hash::sha256::Sha256;
///
/// let hash = Sha256::digest(b"hello world");
/// assert_eq!(hex(&hash), "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
///
/// fn hex(bytes: &[u8]) -> String {
///     use std::fmt::Write;
///     bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
///         let _ = write!(acc, "{b:02x}");
///         acc
///     })
/// }
/// ```
pub struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buffer_len: usize,
    total_len: u64,
}

// Initial hash values (FIPS 180-4 §5.3.3)
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

// Round constants (FIPS 180-4 §4.2.2)
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

// SHA-256 logical functions (FIPS 180-4 §4.1.2)
#[inline]
const fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

#[inline]
const fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

#[inline]
const fn big_sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

#[inline]
const fn big_sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

#[inline]
const fn small_sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

#[inline]
const fn small_sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

impl Sha256 {
    /// Create a new SHA-256 hasher.
    pub fn new() -> Self {
        Self {
            state: H0,
            buffer: [0u8; 64],
            buffer_len: 0,
            total_len: 0,
        }
    }

    /// Feed data into the hasher.
    pub fn update(&mut self, data: &[u8]) {
        self.total_len += data.len() as u64;
        let mut offset = 0;

        // Fill buffer if partial
        if self.buffer_len > 0 {
            let space = 64 - self.buffer_len;
            let to_copy = data.len().min(space);
            self.buffer[self.buffer_len..self.buffer_len + to_copy]
                .copy_from_slice(&data[..to_copy]);
            self.buffer_len += to_copy;
            offset = to_copy;

            if self.buffer_len == 64 {
                let block = self.buffer;
                self.compress(&block);
                self.buffer_len = 0;
            }
        }

        // Process full blocks
        while offset + 64 <= data.len() {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[offset..offset + 64]);
            self.compress(&block);
            offset += 64;
        }

        // Buffer remaining
        if offset < data.len() {
            let remaining = data.len() - offset;
            self.buffer[..remaining].copy_from_slice(&data[offset..]);
            self.buffer_len = remaining;
        }
    }

    /// Finalize and return the digest.
    pub fn finalize(mut self) -> Sha256Digest {
        // Padding (FIPS 180-4 §5.1.1)
        let bit_len = self.total_len * 8;
        self.update(&[0x80]);

        // Pad with zeros until 56 mod 64
        while self.buffer_len != 56 {
            self.update(&[0x00]);
        }

        // Append length as big-endian 64-bit
        self.update(&bit_len.to_be_bytes());

        // Convert state to bytes
        let mut output = [0u8; 32];
        for (i, word) in self.state.iter().enumerate() {
            output[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
        }
        output
    }

    /// One-shot digest computation.
    pub fn digest(data: &[u8]) -> Sha256Digest {
        let mut hasher = Self::new();
        hasher.update(data);
        hasher.finalize()
    }

    /// Compress a single 512-bit block (FIPS 180-4 §6.2.2).
    #[allow(clippy::many_single_char_names)]
    fn compress(&mut self, block: &[u8; 64]) {
        // Prepare message schedule W
        let mut w = [0u32; 64];
        for t in 0..16 {
            w[t] = u32::from_be_bytes([
                block[t * 4],
                block[t * 4 + 1],
                block[t * 4 + 2],
                block[t * 4 + 3],
            ]);
        }
        for t in 16..64 {
            w[t] = small_sigma1(w[t - 2])
                .wrapping_add(w[t - 7])
                .wrapping_add(small_sigma0(w[t - 15]))
                .wrapping_add(w[t - 16]);
        }

        // Initialize working variables
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;

        // 64 rounds
        for t in 0..64 {
            let t1 = h
                .wrapping_add(big_sigma1(e))
                .wrapping_add(ch(e, f, g))
                .wrapping_add(K[t])
                .wrapping_add(w[t]);
            let t2 = big_sigma0(a).wrapping_add(maj(a, b, c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        // Update state
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a digest as lowercase hex string.
pub fn hex_digest(digest: &Sha256Digest) -> String {
    use std::fmt::Write;
    digest.iter().fold(String::with_capacity(64), |mut acc, b| {
        let _ = write!(acc, "{b:02x}");
        acc
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        use std::fmt::Write;
        bytes
            .iter()
            .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
                let _ = write!(acc, "{b:02x}");
                acc
            })
    }

    // NIST CAVP test vectors (FIPS 180-4 examples)
    #[test]
    fn nist_short_msg_abc() {
        // "abc" → known SHA-256 hash
        let hash = Sha256::digest(b"abc");
        assert_eq!(
            hex(&hash),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn nist_short_msg_empty() {
        let hash = Sha256::digest(b"");
        assert_eq!(
            hex(&hash),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn nist_two_block() {
        // "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
        let hash = Sha256::digest(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(
            hex(&hash),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn nist_long_msg() {
        // "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"
        let hash = Sha256::digest(
            b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
        );
        assert_eq!(
            hex(&hash),
            "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"
        );
    }

    #[test]
    fn incremental_same_as_oneshot() {
        let data = b"Hello, World! This is a test of incremental hashing.";
        let oneshot = Sha256::digest(data);

        let mut hasher = Sha256::new();
        hasher.update(&data[..13]);
        hasher.update(&data[13..26]);
        hasher.update(&data[26..]);
        let incremental = hasher.finalize();

        assert_eq!(oneshot, incremental);
    }

    #[test]
    fn byte_at_a_time() {
        let data = b"test data for byte-by-byte";
        let oneshot = Sha256::digest(data);

        let mut hasher = Sha256::new();
        for &byte in data.iter() {
            hasher.update(&[byte]);
        }
        let result = hasher.finalize();

        assert_eq!(oneshot, result);
    }

    #[test]
    fn hello_world() {
        let hash = Sha256::digest(b"hello world");
        assert_eq!(
            hex(&hash),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn hex_digest_format() {
        let hash = Sha256::digest(b"");
        let formatted = hex_digest(&hash);
        assert_eq!(formatted.len(), 64);
        assert!(formatted.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
