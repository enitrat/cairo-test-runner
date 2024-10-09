use anyhow::Result;
use test_runner::manual_types::MyStruct;
use test_runner::manual_types::{Stack, U128, U32};
use test_runner::test_utils::load_and_run_cairo_function;

use proptest::prelude::*;

fn reference_bytes32_words(input: u128) -> u128 {
    (input + 31) / 32
}

#[test]
fn test_bytes32_words() -> Result<()> {
    let test_cases = vec![
        ("10", 1),
        ("32", 1),
        ("33", 2),
        ("64", 2),
        ("65", 3),
        ("100", 4),
    ];

    for (input, expected) in test_cases {
        let args = format!("[{}]", input);
        let result: U32 = load_and_run_cairo_function("bytes32_words", &args)?;
        assert_eq!(expected, *result);
    }

    Ok(())
}

#[test]
fn test_my_struct() -> Result<()> {
    let args = "[1, 2]";
    let result = load_and_run_cairo_function::<MyStruct>("my_struct", args)?;
    let expected = MyStruct {
        field_0: 1,
        field_1: 2,
    };
    assert_eq!(expected, result);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn test_bytes32_words_prop(input in 0u128..=u128::MAX) {
        let expected = reference_bytes32_words(input);
        let args = format!("[{}]", input);
        let result = load_and_run_cairo_function::<U128>("bytes32_words", &args).unwrap();

        prop_assert_eq!(expected, *result);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_stack_push(input in prop::collection::vec(0u128..=u128::MAX, 0..10), pushed_value in 0u128..=u128::MAX) {
        let args = format!("[{:?}, {}]", input, pushed_value);
    let result: Stack = load_and_run_cairo_function("stack_push_should_add_element", &args).unwrap();

        let mut expected_push = input.clone();
        expected_push.push(pushed_value);
        let expected_push = Stack::from_array(expected_push);

        prop_assert_eq!(result, expected_push);
    }

    #[test]
    fn test_stack_pop(input in prop::collection::vec(0u128..=u128::MAX, 1..10)) {
        let args = format!("[{:?}]", input);
        let result: Stack = load_and_run_cairo_function("stack_pop_should_remove_last_element", &args).unwrap();

        let mut expected = input.clone();
        expected.pop();
        let expected = Stack::from_array(expected);

        prop_assert_eq!(result, expected);
    }

    #[test]
    fn test_stack_pop_return(input in prop::collection::vec(0u128..=u128::MAX, 1..10)) {
        let args = format!("[{:?}]", input);
        let result: U128 = load_and_run_cairo_function("stack_pop_should_return_last_element", &args).unwrap();

        prop_assert_eq!(result, U128(*input.last().unwrap()));
    }
}
