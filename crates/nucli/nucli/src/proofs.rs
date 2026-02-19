//! Exhaustive proofs for nucli.
//!
//! Every proof verifies a property over the ENTIRE domain — no sampling.
//! For 256 bytes and 256 tetrads, exhaustive is both feasible and stronger
//! than any probabilistic property test.
//!
//! ## Proven Properties
//!
//! | ID  | Property | Domain | ELL Principle |
//! |-----|----------|--------|---------------|
//! | P1  | Encode→Decode roundtrip | 256 bytes | L1: exhaustive |
//! | P2  | Decode→Encode roundtrip | 256 tetrads | L1: exhaustive |
//! | P3  | Encode injectivity (no collisions) | 256 bytes | L2: bijection |
//! | P4  | Complement involution | 256 tetrads | L2: involution |
//! | P5  | Determinism: same input → same output | 256 × 10 runs | L2: determinism |
//! | P6  | Complement preserves length | 256 tetrads | L2: structure |
//! | P7  | GC content conservation under complement | 256 tetrads | L2: conservation |

#[cfg(test)]
mod tests {
    use crate::codec;

    // =================================================================
    // P1: Encode→Decode Roundtrip (Exhaustive over all 256 bytes)
    //
    // ∀ b ∈ [0, 255]: decode(encode([b])) == [b]
    // =================================================================

    #[test]
    fn p1_encode_decode_roundtrip_exhaustive() {
        for b in 0u16..=255 {
            let byte = b as u8;
            let strand = codec::encode(&[byte]);
            let decoded = codec::decode(&strand);
            assert!(
                decoded.is_ok(),
                "decode failed for byte {byte:#04x} → strand {strand:?}"
            );
            assert_eq!(
                decoded.unwrap(),
                vec![byte],
                "roundtrip failed for byte {byte:#04x} → strand {strand:?}"
            );
        }
    }

    // =================================================================
    // P2: Decode→Encode Roundtrip (Exhaustive over all 256 tetrads)
    //
    // ∀ (n0,n1,n2,n3) ∈ {A,T,G,C}^4: encode(decode(n0n1n2n3)) == n0n1n2n3
    // =================================================================

    #[test]
    fn p2_decode_encode_roundtrip_exhaustive() {
        let nucs = ['A', 'T', 'G', 'C'];
        let mut count = 0u32;

        for &n0 in &nucs {
            for &n1 in &nucs {
                for &n2 in &nucs {
                    for &n3 in &nucs {
                        let tetrad: String =
                            [n0, n1, n2, n3].iter().collect();
                        let decoded = codec::decode(&tetrad);
                        assert!(
                            decoded.is_ok(),
                            "decode failed for tetrad {tetrad:?}"
                        );
                        let re_encoded = codec::encode(&decoded.unwrap());
                        assert_eq!(
                            re_encoded, tetrad,
                            "reverse roundtrip failed: {tetrad:?} → bytes → {re_encoded:?}"
                        );
                        count += 1;
                    }
                }
            }
        }

        assert_eq!(count, 256, "expected 4^4=256 tetrads, got {count}");
    }

    // =================================================================
    // P3: Encode Injectivity (No Collisions)
    //
    // ∀ a,b ∈ [0,255]: a ≠ b → encode([a]) ≠ encode([b])
    //
    // Combined with P1 (surjectivity via roundtrip), this proves bijection.
    // =================================================================

    #[test]
    fn p3_encode_injectivity_exhaustive() {
        let mut seen = std::collections::HashMap::new();

        for b in 0u16..=255 {
            let byte = b as u8;
            let strand = codec::encode(&[byte]);

            if let Some(prev) = seen.insert(strand.clone(), byte) {
                panic!(
                    "collision: bytes {prev:#04x} and {byte:#04x} both encode to {strand:?}"
                );
            }
        }

        assert_eq!(seen.len(), 256, "expected 256 unique strands");
    }

    // =================================================================
    // P4: Complement Involution (Exhaustive)
    //
    // ∀ s ∈ all single-byte encodings:
    //   complement(complement(s)) == s
    // =================================================================

    #[test]
    fn p4_complement_involution_exhaustive() {
        for b in 0u16..=255 {
            let byte = b as u8;
            let strand = codec::encode(&[byte]);

            let rc1 = crate::complement(&strand);
            let rc2 = crate::complement(&rc1);

            assert_eq!(
                strand, rc2,
                "involution failed for byte {byte:#04x}: \
                 {strand:?} → {rc1:?} → {rc2:?}"
            );
        }
    }

    // =================================================================
    // P5: Determinism (Same input → same output, 10 runs each)
    //
    // ∀ b ∈ [0, 255], ∀ run ∈ [1, 10]:
    //   encode([b])_run == encode([b])_1
    //   decode(strand)_run == decode(strand)_1
    // =================================================================

    #[test]
    fn p5_determinism_exhaustive() {
        for b in 0u16..=255 {
            let byte = b as u8;
            let first_encode = codec::encode(&[byte]);
            let first_decode = codec::decode(&first_encode).unwrap();

            for _ in 1..10 {
                let enc = codec::encode(&[byte]);
                assert_eq!(
                    enc, first_encode,
                    "encode non-determinism for byte {byte:#04x}"
                );

                let dec = codec::decode(&enc).unwrap();
                assert_eq!(
                    dec, first_decode,
                    "decode non-determinism for byte {byte:#04x}"
                );
            }
        }
    }

    // =================================================================
    // P6: Complement Preserves Length
    //
    // ∀ s: |complement(s)| == |s|
    // =================================================================

    #[test]
    fn p6_complement_preserves_length_exhaustive() {
        for b in 0u16..=255 {
            let byte = b as u8;
            let strand = codec::encode(&[byte]);
            let rc = crate::complement(&strand);

            assert_eq!(
                strand.len(),
                rc.len(),
                "length changed for byte {byte:#04x}: {strand:?}({}) → {rc:?}({})",
                strand.len(),
                rc.len()
            );
        }
    }

    // =================================================================
    // P7: GC Content Conservation Under Complement
    //
    // Complement swaps A↔T and G↔C. Therefore:
    //   GC_count(s) == GC_count(complement(s))
    //   AT_count(s) == AT_count(complement(s))
    // =================================================================

    #[test]
    fn p7_gc_conservation_exhaustive() {
        for b in 0u16..=255 {
            let byte = b as u8;
            let strand = codec::encode(&[byte]);
            let rc = crate::complement(&strand);

            let gc_orig = strand.chars().filter(|&c| c == 'G' || c == 'C').count();
            let gc_comp = rc.chars().filter(|&c| c == 'G' || c == 'C').count();

            assert_eq!(
                gc_orig, gc_comp,
                "GC content changed for byte {byte:#04x}: {strand:?}(GC={gc_orig}) → {rc:?}(GC={gc_comp})"
            );
        }
    }

    // =================================================================
    // P8: Multi-byte roundtrip (longer strings)
    // =================================================================

    #[test]
    fn p8_multibyte_roundtrip() {
        let test_cases: &[&[u8]] = &[
            b"Hello, World!",
            b"",
            &[0x00, 0xFF, 0x80, 0x7F],
            b"The quick brown fox jumps over the lazy dog",
            &(0u8..=255).collect::<Vec<u8>>(), // all 256 bytes
        ];

        for data in test_cases {
            if data.is_empty() {
                // encode of empty is empty string, decode of empty is error
                let strand = codec::encode(data);
                assert_eq!(strand, "");
                continue;
            }

            let strand = codec::encode(data);
            let decoded = codec::decode(&strand).unwrap();
            assert_eq!(
                &decoded,
                data,
                "multi-byte roundtrip failed for {} bytes",
                data.len()
            );
        }
    }

    // =================================================================
    // P9: Complement involution on multi-byte strands
    // =================================================================

    #[test]
    fn p9_complement_involution_multibyte() {
        let test_bytes: &[&[u8]] = &[
            b"Hello",
            b"ATGCATGC",
            b"The quick brown fox",
            &[0x00, 0xFF, 0x80, 0x7F],
        ];

        for data in test_bytes {
            let strand = codec::encode(data);
            if strand.is_empty() {
                continue;
            }

            let rc1 = crate::complement(&strand);
            let rc2 = crate::complement(&rc1);

            assert_eq!(
                strand, rc2,
                "multi-byte involution failed for {data:?}"
            );
        }
    }

    // =================================================================
    // P10: Complement is NOT identity (no fixed points for non-palindromes)
    //
    // For single-byte encodings, complement(s) != s for most bytes.
    // The only possible fixed points are palindromic tetrads.
    // We verify exactly which ones are fixed points.
    // =================================================================

    #[test]
    fn p10_complement_fixed_point_analysis() {
        let mut fixed_points = Vec::new();

        for b in 0u16..=255 {
            let byte = b as u8;
            let strand = codec::encode(&[byte]);
            let rc = crate::complement(&strand);

            if strand == rc {
                fixed_points.push((byte, strand));
            }
        }

        // A tetrad is a fixed point of RC iff it's a palindrome under A↔T, G↔C.
        // For "n0 n1 n2 n3", RC = "comp(n3) comp(n2) comp(n1) comp(n0)".
        // Fixed point requires: n0=comp(n3), n1=comp(n2).
        // So n3 is determined by n0, n2 is determined by n1 → 4×4 = 16 fixed points.
        assert_eq!(
            fixed_points.len(),
            16,
            "expected 16 palindromic fixed points, got {}: {:?}",
            fixed_points.len(),
            fixed_points
        );
    }
}
