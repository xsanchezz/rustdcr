#[cfg(test)]
mod amount {
    use crate::dcrutil::amount;

    #[test]
    fn test_amount_creation() {
        #[derive(Default)]
        pub struct Test {
            pub name: String,
            pub amount: f64,
            pub valid: bool,
            pub expected: crate::dcrutil::amount::Amount,
        }

        let tests = vec![
            Test {
                name: "zero".to_string(),
                amount: 0.0,
                valid: true,
                expected: amount::Amount(0),
            },
            Test {
                name: "max producable".to_string(),
                amount: 21e6,
                valid: true,
                expected: amount::Amount(amount::constants::MAX_AMOUNT as i64),
            },
            Test {
                name: "min producable".to_string(),
                amount: -21e6,
                valid: true,
                expected: amount::Amount(-amount::constants::MAX_AMOUNT as i64),
            },
            Test {
                name: "exceeds max producable".to_string(),
                amount: 21e6 + 1e-8,
                valid: true,
                expected: amount::Amount(amount::constants::MAX_AMOUNT as i64 + 1),
            },
            Test {
                name: "exceeds min producable".to_string(),
                amount: -21e6 - 1e-8,
                valid: true,
                expected: amount::Amount(-amount::constants::MAX_AMOUNT as i64 - 1),
            },
            Test {
                name: "one hundred".to_string(),
                amount: 100.0,
                valid: true,
                expected: amount::Amount(amount::constants::ATOMS_PER_COIN as i64 * 100),
            },
            Test {
                name: "fraction".to_string(),
                amount: 0.01234567,
                valid: true,
                expected: amount::Amount(1234567),
            },
            Test {
                name: "rounding up".to_string(),
                amount: 54.999999999999943157,
                valid: true,
                expected: amount::Amount(55 * amount::constants::ATOMS_PER_COIN as i64),
            },
            Test {
                name: "rounding down".to_string(),
                amount: 55.000000000000056843,
                valid: true,
                expected: amount::Amount(55 * amount::constants::ATOMS_PER_COIN as i64),
            },
            // Negative test.
            Test {
                name: "not-a-number".to_string(),
                amount: std::f64::NAN,
                valid: false,

                ..Default::default()
            },
            Test {
                name: "-infinity".to_string(),
                amount: std::f64::NEG_INFINITY,
                valid: false,

                ..Default::default()
            },
            Test {
                name: "+infinity".to_string(),
                amount: std::f64::INFINITY,
                valid: false,

                ..Default::default()
            },
        ];

        for test in tests {
            match amount::new(test.amount) {
                Ok(e) => {
                    if !test.valid {
                        panic!(
                            "{}: Invalid amount test passed, amount: {}",
                            test.name,
                            e.to_string()
                        );
                    }

                    if e != test.expected {
                        panic!(
                            "{}: created amount {} does not match expected {}",
                            test.name,
                            e.to_string(),
                            test.expected.to_string()
                        )
                    }
                }

                Err(e) => {
                    if test.valid {
                        panic!("{}: valid amount test failed with error: {}", test.name, e);
                    }

                    continue;
                }
            };
        }
    }

    #[test]
    fn test_amount_unit_conversion() {
        pub struct Test<'a> {
            pub name: &'a str,
            pub amount: crate::dcrutil::amount::Amount,
            pub denomination: crate::dcrutil::amount::Denomination,
            pub converted: f64,
            pub amount_in_string: &'a str,
        }

        let tests = vec![
            Test {
                name: "MDCR",
                amount: amount::Amount(amount::constants::MAX_AMOUNT as i64),
                denomination: amount::Denomination::AmountMegaCoin,
                converted: 21.0,
                amount_in_string: "21 MDCR",
            },
            Test {
                name: "kDCR",
                amount: amount::Amount(44433322211100),
                denomination: amount::Denomination::AmountKiloCoin,
                converted: 444.33322211100,
                amount_in_string: "444.333222111 kDCR",
            },
            Test {
                name: "Coin",
                amount: amount::Amount(44433322211100),
                denomination: amount::Denomination::AmountCoin,
                converted: 444333.22211100,
                amount_in_string: "444333.222111 DCR",
            },
            Test {
                name: "mDCR",
                amount: amount::Amount(44433322211100),
                denomination: amount::Denomination::AmountMilliCoin,
                converted: 444333222.11100,
                amount_in_string: "444333222.111 mDCR",
            },
            Test {
                name: "μDCR",
                amount: amount::Amount(44433322211100),
                denomination: amount::Denomination::AmountMicroCoin,
                converted: 444333222111.00,
                amount_in_string: "444333222111 μDCR",
            },
            Test {
                name: "μDCR",
                amount: amount::Amount(44433322211100),
                denomination: amount::Denomination::AmountMicroCoin,
                converted: 444333222111.00,
                amount_in_string: "444333222111 μDCR",
            },
            Test {
                name: "atom",
                amount: amount::Amount(44433322211100),
                denomination: amount::Denomination::AmountAtom,
                converted: 44433322211100.00,
                amount_in_string: "44433322211100 Atom",
            },
            Test {
                name: "non-standard unit",
                amount: amount::Amount(44433322211100),
                denomination: amount::Denomination::AmountAtom,
                converted: 44433322211100.00,
                amount_in_string: "44433322211100 Atom",
            },
        ];

        for test in tests {
            let amount_number = test.amount.to_unit(test.denomination);
            if amount_number != test.converted {
                panic!(
                    "{}: converted value in number {} does not match expected {}",
                    test.name, amount_number, test.converted
                )
            }

            let amount_string = test.amount.format(test.denomination);
            if amount_string != test.amount_in_string {
                panic!(
                    "{}: converted value in string {} does not match expected {}",
                    test.name, amount_string, test.amount_in_string
                )
            }

            // Verify that Amount.to_coin works as advertised.
            let f1 = test.amount.to_unit(amount::Denomination::AmountCoin);
            let f2 = test.amount.to_coin();
            if f1 != f2 {
                panic!(
                    "{}: to_coin does not match to_unit(AmountCoin): {} != {}",
                    test.name, f1, f2
                )
            }

            // Verify that Amount.String works as advertised.
            let s1 = test.amount.format(amount::Denomination::AmountCoin);
            let s2 = test.amount.to_string();
            if s1 != s2 {
                panic!(
                    "{}: String does not match Format(AmountCoin): {} != {}",
                    test.name, s1, s2
                )
            }
        }
    }

    #[test]
    fn test_amount_mulf64() {
        pub struct Test<'a> {
            pub name: &'a str,
            pub amount: crate::dcrutil::amount::Amount,
            pub multiply_by: f64,
            pub result: crate::dcrutil::amount::Amount,
        }

        let tests = vec![
            Test {
                name: "Multiply 0.1 DCR by 2",
                amount: crate::dcrutil::amount::Amount(100e5 as i64), // 0.1 DCR
                multiply_by: 2.0,
                result: crate::dcrutil::amount::Amount(200e5 as i64), // 0.2 DCR
            },
            Test {
                name: "Multiply 0.2 DCR by 0.02",
                amount: crate::dcrutil::amount::Amount(200e5 as i64), // 0.2 DCR
                multiply_by: 1.02,
                result: crate::dcrutil::amount::Amount(204e5 as i64), // 0.204 DCR
            },
            Test {
                name: "Multiply 0.1 DCR by -2",
                amount: crate::dcrutil::amount::Amount(100e5 as i64), // 0.1 DCR
                multiply_by: -2.0,
                result: crate::dcrutil::amount::Amount(-200e5 as i64), // -0.2 DCR
            },
            Test {
                name: "Multiply 0.2 DCR by -0.02",
                amount: crate::dcrutil::amount::Amount(200e5 as i64), // 0.2 DCR
                multiply_by: -1.02,
                result: crate::dcrutil::amount::Amount(-204e5 as i64), // -0.204 DCR
            },
            Test {
                name: "Multiply -0.1 DCR by 2",
                amount: crate::dcrutil::amount::Amount(-100e5 as i64), // -0.1 DCR
                multiply_by: 2.0,
                result: crate::dcrutil::amount::Amount(-200e5 as i64), // -0.2 DCR
            },
            Test {
                name: "Multiply -0.2 DCR by 0.02",
                amount: crate::dcrutil::amount::Amount(-200e5 as i64), // -0.2 DCR
                multiply_by: 1.02,
                result: crate::dcrutil::amount::Amount(-204e5 as i64), // -0.204 DCR
            },
            Test {
                name: "Multiply -0.1 DCR by -2",
                amount: crate::dcrutil::amount::Amount(-100e5 as i64), // -0.1 DCR
                multiply_by: -2.0,
                result: crate::dcrutil::amount::Amount(200e5 as i64), // 0.2 DCR
            },
            Test {
                name: "Multiply -0.2 DCR by -0.02",
                amount: crate::dcrutil::amount::Amount(-200e5 as i64), // -0.2 DCR
                multiply_by: -1.02,
                result: crate::dcrutil::amount::Amount(204e5 as i64), // 0.204 DCR
            },
            Test {
                name: "Round down",
                amount: crate::dcrutil::amount::Amount(49 as i64), // 49 Atoms
                multiply_by: 0.01,
                result: crate::dcrutil::amount::Amount(0 as i64),
            },
            Test {
                name: "Round up",
                amount: crate::dcrutil::amount::Amount(50 as i64), // 50 Atoms
                multiply_by: 0.01,
                result: crate::dcrutil::amount::Amount(1 as i64), // 1 Atom
            },
            Test {
                name: "Multiply by 0.",
                amount: crate::dcrutil::amount::Amount(1e8 as i64), // 1 DCR
                multiply_by: 0.0,
                result: crate::dcrutil::amount::Amount(0 as i64), // 0 DCR
            },
            Test {
                name: "Multiply 1 by 0.5",
                amount: crate::dcrutil::amount::Amount(1 as i64), // 1 Atoms
                multiply_by: 0.5,
                result: crate::dcrutil::amount::Amount(1 as i64), // 1 DCR
            },
            Test {
                name: "Multiply 100 by 66%",
                amount: crate::dcrutil::amount::Amount(100 as i64), // 100 Atoms
                multiply_by: 0.66,
                result: crate::dcrutil::amount::Amount(66 as i64), // 66 DCR
            },
            Test {
                name: "Multiply 100 by 66.6%",
                amount: crate::dcrutil::amount::Amount(100 as i64), // 100 Atoms
                multiply_by: 0.666,
                result: crate::dcrutil::amount::Amount(67 as i64), // 67 Atoms
            },
            Test {
                name: "Multiply 100 by 2/3",
                amount: crate::dcrutil::amount::Amount(100 as i64), // 100 Atoms
                multiply_by: 2.0 / 3.0,
                result: crate::dcrutil::amount::Amount(67 as i64), // 67 Atoms
            },
        ];

        for test in tests {
            let amount = test.amount.mul_f64(test.multiply_by);
            if amount != test.result {
                panic!(
                    "{}: expected {} got {}",
                    test.name,
                    test.amount.to_string(),
                    amount.to_string()
                );
            }
        }
    }

    #[test]
    fn test_amount_sorter() {
        struct Test<'a> {
            name: &'a str,
            unsorted: Vec<amount::Amount>,
            sorted: Vec<amount::Amount>,
        }

        let tests = vec![
            Test {
                name: "Sort zero length slice of Amounts",
                unsorted: vec![],
                sorted: vec![],
            },
            Test {
                name: "Sort 1-element slice of Amounts",
                unsorted: vec![amount::Amount(7)],
                sorted: vec![amount::Amount(7)],
            },
            Test {
                name: "Sort 2-element slice of Amounts",
                unsorted: vec![amount::Amount(7), amount::Amount(5)],
                sorted: vec![amount::Amount(5), amount::Amount(7)],
            },
            Test {
                name: "Sort 6-element slice of Amounts",
                unsorted: vec![
                    amount::Amount(0),
                    amount::Amount(9e8 as i64),
                    amount::Amount(4e6 as i64),
                    amount::Amount(4e6 as i64),
                    amount::Amount(3 as i64),
                    amount::Amount(9e12 as i64),
                ],
                sorted: vec![
                    amount::Amount(0),
                    amount::Amount(3 as i64),
                    amount::Amount(4e6 as i64),
                    amount::Amount(4e6 as i64),
                    amount::Amount(9e8 as i64),
                    amount::Amount(9e12 as i64),
                ],
            },
        ];

        for test in tests {
            let mut sorted = test.unsorted;
            sorted.sort();

            if sorted != test.sorted {
                panic!(
                    "AmountSort {} got {:?} want {:?}",
                    test.name, sorted, test.sorted
                )
            }
        }
    }
}
