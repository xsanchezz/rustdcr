// Note: All test data is taken from the Bitcoin blockchain.  This data is
// intentionally unmodified since it would be an unnecessary difference between
// the dcrd and btcd codebases.

// mainNetGenesisHash is the hash of the first block in the block chain for the
// main network (genesis block).

#[cfg(test)]
mod chain_hash {
    // Note: All test data is taken from the Bitcoin blockchain.  This data is
    // intentionally unmodified since it would be an unnecessary difference between
    // the dcrd and btcd codebases.

    use crate::chaincfg::chainhash::{constants::HASH_SIZE, Hash};

    // mainNetGenesisHash is the hash of the first block in the block chain for the
    // main network (genesis block).
    const MAIN_NET_GENESIS_HASH: [u8; HASH_SIZE] = [
        0x6f, 0xe2, 0x8c, 0x0a, 0xb6, 0xf1, 0xb3, 0x72, 0xc1, 0xa6, 0xa2, 0x46, 0xae, 0x63, 0xf7,
        0x4f, 0x93, 0x1e, 0x83, 0x65, 0xe1, 0x5a, 0x08, 0x9c, 0x68, 0xd6, 0x19, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    #[test]
    // TestHash tests the Hash API.
    fn test_hash() {
        // Hash of block 234439.
        let hash_str =
            match Hash::new_from_str("14a0810ac680a3eb3f82edc878cea25ec41d6b790744e5daeef") {
                Ok(e) => e,

                Err(e) => {
                    panic!("NewHashFromStr: : {:?}", e);
                }
            };

        // Hash of block 234440 as byte slice.
        let buf: [u8; HASH_SIZE] = [
            0x79, 0xa6, 0x1a, 0xdb, 0xc6, 0xe5, 0xa2, 0xe1, 0x39, 0xd2, 0x71, 0x3a, 0x54, 0x6e,
            0xc7, 0xc8, 0x75, 0x63, 0x2e, 0x75, 0xf1, 0xdf, 0x9c, 0x3f, 0xa6, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let mut hash = match Hash::new(buf.to_vec()) {
            Ok(e) => e,

            Err(e) => panic!("NewHash: unexpected error: {:?}", e),
        };

        // Ensure proper size.
        assert_eq!(HASH_SIZE, hash.len(), "NewHash: hash length mismatch.");

        // Ensure contents match.
        assert_eq!(buf, *hash.bytes(), "NewHash: hash contents mismatch");

        // Ensure contents of hash of block 234440 don't match 234439.
        if hash.is_equal(&hash_str) {
            panic!(
                "IsEqual: hash contents should not match - got: {:?}, want: {:?}",
                hash, hash_str
            )
        }

        // Set hash from byte slice and ensure contents match.
        hash.set_bytes(hash_str.bytes().to_vec())
            .expect("SetBytes: ");

        if !hash.is_equal(&hash_str) {
            panic!(
                "IsEqual: hash contents mismatch - got: {:?}, want: {:?}",
                hash, hash_str
            )
        }

        // ToDo: Is there an need for a None Hash?

        // Invalid size for SetBytes.
        hash.set_bytes([0x00].to_vec())
            .expect_err("SetBytes: failed to received expected err - got: nil");

        // Invalid size for NewHash.
        let invalid_hash = Vec::with_capacity(HASH_SIZE + 1);
        Hash::new(invalid_hash).expect_err("NewHash: failed to received expected err - got: nil");
    }

    #[test]
    fn test_hash_string() {
        let want_str = "000000000003ba27aa200b1cecaad478d2b00432346c3f1f3986da1afd33e506";

        let buf: [u8; HASH_SIZE] = [
            0x06, 0xe5, 0x33, 0xfd, 0x1a, 0xda, 0x86, 0x39, 0x1f, 0x3f, 0x6c, 0x34, 0x32, 0x04,
            0xb0, 0xd2, 0x78, 0xd4, 0xaa, 0xec, 0x1c, 0x0b, 0x20, 0xaa, 0x27, 0xba, 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let hash = match Hash::new(buf.to_vec()) {
            Ok(e) => e,

            Err(e) => panic!("ChainHash: {}", e),
        };

        println!("{:?}", hash.string());

        assert_eq!(want_str.to_string(), hash.string().unwrap());
    }

    struct Test {
        pub hash_str: String,
        pub want: [u8; HASH_SIZE],
        pub err: Result<(), crate::chaincfg::chainhash::error::ChainHashError>,
    }

    #[test]
    fn test_new_hash_from_str() {
        let tests = vec![
            Test {
                hash_str: String::from(
                    "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
                ),
                want: super::chain_hash::MAIN_NET_GENESIS_HASH,
                err: Ok(()),
            },
            Test {
                hash_str: String::from("19d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"),
                want: super::chain_hash::MAIN_NET_GENESIS_HASH,
                err: Ok(()),
            },
            Test {
                hash_str: String::from(""),
                want: [0; HASH_SIZE],
                err: Ok(()),
            },
            Test {
                hash_str: String::from("1"),
                want: [
                    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                err: Ok(()),
            },
            Test {
                hash_str: String::from("3264bc2ac36a60840790ba1d475d01367e7c723da941069e9dc"),
                want: [
                    0xdc, 0xe9, 0x69, 0x10, 0x94, 0xda, 0x23, 0xc7, 0xe7, 0x67, 0x13, 0xd0, 0x75,
                    0xd4, 0xa1, 0x0b, 0x79, 0x40, 0x08, 0xa6, 0x36, 0xac, 0xc2, 0x4b, 0x26, 0x03,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                err: Ok(()),
            },
            Test {
                hash_str: String::from(
                    "01234567890123456789012345678901234567890123456789012345678912345",
                ),
                want: [0; HASH_SIZE],
                err: Err(crate::chaincfg::chainhash::ChainHashError::HashStringSize),
            },
            Test {
                hash_str: String::from("abcdefg"),
                want: [0; HASH_SIZE],
                err: Err(crate::chaincfg::chainhash::ChainHashError::HexDecode(
                    hex::FromHexError::InvalidHexCharacter { c: 'g', index: 7 },
                )),
            },
            Test {
                hash_str: String::from("banana"),
                want: [0; HASH_SIZE],
                err: Err(crate::chaincfg::chainhash::ChainHashError::HexDecode(
                    // ToDo: There is an error here, we should instead have index as 2.
                    // See https://github.com/KokaKiwi/rust-hex/issues/49.
                    hex::FromHexError::InvalidHexCharacter { c: 'n', index: 2 },
                )),
            },
        ];

        log::info!(
            "Running {} tests in test_new_hash_from_str function",
            tests.len()
        );

        for (i, test) in tests.iter().enumerate() {
            match Hash::new_from_str(test.hash_str.as_str()) {
                // If test is an Ok, check if equal.
                Ok(e) => assert_eq!(
                    test.want,
                    *e.bytes(),
                    "Failed to assert equal Ok values, index: {}",
                    i
                ),

                Err(err) => match test.err.as_ref() {
                    Ok(_) => panic!(
                        "Expected case returned OK test returned error: {}. Index: {}",
                        err, i
                    ),

                    Err(case_err) => {
                        assert_eq!(
                            case_err.to_string(),
                            err.to_string(),
                            "Failed to assert equal errors, index: {}",
                            i
                        );
                    }
                },
            }
        }
    }
}
