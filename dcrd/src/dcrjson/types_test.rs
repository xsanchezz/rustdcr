#[cfg(test)]
mod dcr_types_test {

    use crate::dcrjson::result_types::{ScriptSig, Vin};

    #[test]
    fn test_chain_svr_custom_results() {
        #[derive(serde::Serialize)]
        struct Value<'a> {
            name: &'a str,
            expected: serde_json::Value,
            result: Vin,
        }

        let tests = vec![
            Value {
                name: "custom vin marshal with coinbase",
                expected: serde_json::json!({
                    "coinbase":"021234",
                    "stakebase":"",
                    "txid": "",
                    "vout": 0,
                    "tree": 0,
                    "sequence":4294967295 as u32,
                    "amountin":0.0,
                    "blockheight": 0,
                    "blockindex":0,
                    "scriptSig": serde_json::Value::Null,
                }),
                result: Vin {
                    coinbase: String::from("021234"),
                    sequence: 4294967295,
                    ..Default::default()
                },
            },
            Value {
                name: "custom vin marshal without coinbase",
                expected: serde_json::json!({
                    "coinbase":"",
                    "stakebase":"",
                    "txid":"123",
                    "vout":1,
                    "tree":0,
                    "sequence":4294967295 as u32,
                    "amountin":0.0,
                    "blockheight":0,
                    "blockindex":0,
                    "scriptSig":{"asm":"0","hex":"00"}}),
                result: Vin {
                    tx_id: String::from("123"),
                    vout: 1,
                    tree: 0,
                    sequence: 4294967295,
                    script_sig: Some(ScriptSig {
                        asm: String::from("0"),
                        hex: String::from("00"),
                    }),
                    ..Default::default()
                },
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let marshalled = serde_json::to_value(&test.result)
                .expect(&format!("test {} {} failed:", i, test.name));

            assert!(
                marshalled.eq(&test.expected),
                "{} {} failed to return expected result: \n {:#?}\n{:#?}",
                i,
                test.name,
                marshalled,
                test.expected
            )
        }
    }
}
