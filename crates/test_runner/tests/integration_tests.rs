use anyhow::Result;
use starknet_types_core::felt::Felt;
use std::str::FromStr;
use test_runner::test_utils::load_and_run_cairo_function;
use test_runner::generated_types::MyStruct;

use proptest::prelude::*;

fn reference_bytes32_words(input: u128) -> u128 {
    (input + 31) / 32
}

#[test]
fn test_bytes32_words() -> Result<()>{
    let test_cases = vec![
        ("10", 1),
        ("32", 1),
        ("33", 2),
        ("64", 2),
        ("65", 3),
        ("100", 4),
    ];

    for (input, expected) in test_cases {
        let expected_felt = Felt::from_str(expected.to_string().as_str()).unwrap();
        let args = format!("[{}]", input);
        let result = load_and_run_cairo_function("bytes32_words", &args)?;
        assert_eq!(
            result[0], expected_felt,
            "For input {}, expected {}, but got {}",
            input, expected, result[0]
        );
    }

    Ok(())
}

#[test]
fn test_my_struct() -> Result<()> {
    let args = "[1, 2]";
    let result = load_and_run_cairo_function("my_struct", &args)?;
    let expected = MyStruct { field_0: 1, field_1: 2 };
    let actual: MyStruct = result.into();
    assert_eq!(expected, actual);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn test_bytes32_words_prop(input in 0u128..=u128::MAX) {
        let expected = reference_bytes32_words(input);
        let args = format!("[{}]", input);
        let result = load_and_run_cairo_function("bytes32_words", &args).unwrap();
        let cairo_result = u128::try_from(result[0]).unwrap();

        prop_assert_eq!(
            expected,
            cairo_result,
            "For input {}, expected {}, but got {}",
            input,
            expected,
            cairo_result
        );
    }
}
