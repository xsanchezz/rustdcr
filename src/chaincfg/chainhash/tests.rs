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
    const MAIN_NET_GENESIS_HASH: [u8; crate::chaincfg::chainhash::constants::HASH_SIZE] = [
        0x6f, 0xe2, 0x8c, 0x0a, 0xb6, 0xf1, 0xb3, 0x72, 0xc1, 0xa6, 0xa2, 0x46, 0xae, 0x63, 0xf7,
        0x4f, 0x93, 0x1e, 0x83, 0x65, 0xe1, 0x5a, 0x08, 0x9c, 0x68, 0xd6, 0x19, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

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
        let buf = [
            0x79, 0xa6, 0x1a, 0xdb, 0xc6, 0xe5, 0xa2, 0xe1, 0x39, 0xd2, 0x71, 0x3a, 0x54, 0x6e,
            0xc7, 0xc8, 0x75, 0x63, 0x2e, 0x75, 0xf1, 0xdf, 0x9c, 0x3f, 0xa6, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let hash = match Hash::new(buf.to_vec()) {
            Ok(e) => e,

            Err(e) => panic!("NewHash: unexpected error: {:?}", e),
        };

        // Ensure proper size.
        assert_ne!(hash.len(), HASH_SIZE, "NewHash: hash length mismatch.");

        // Ensure contents match.
        assert_ne!(*hash.bytes(), buf, "NewHash: hash contents mismatch");

        // Ensure contents of hash of block 234440 don't match 234439.
        if hash.is_equal(&hash_str) {
            panic!(
                "IsEqual: hash contents should not match - got: {:?}, want: {:?}",
                hash, hash_str
            )
        }

        // Set hash from byte slice and ensure contents match.
        hash.set_bytes(hash_str.clone_hash().bytes().to_vec())
            .expect("SetBytes: ");

        if !hash.is_equal(&hash_str) {
            panic!(
                "IsEqual: hash contents mismatch - got: {:?}, want: {:?}",
                hash, hash_str
            )
        }

        // MustDo: Is there an need for a None Hash?

        // Invalid size for SetBytes.
        hash.set_bytes([0x00].to_vec())
            .expect_err("SetBytes: failed to received expected err - got: nil");

        // Invalid size for NewHash.
        let invalid_hash = Vec::with_capacity(HASH_SIZE + 1);
        Hash::new(invalid_hash).expect_err("NewHash: failed to received expected err - got: nil");
    }
}
