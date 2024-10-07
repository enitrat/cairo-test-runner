use core::num::traits::SaturatingAdd;


/// Returns the amount of 32-bytes words required to represent `x` bytes.
///
/// # Examples
///
/// ```
/// let x = 10;
/// let result = bytes32_words(x);
/// assert_eq!(result, 1);
/// ```
///
/// ```
/// let x = 100;
/// let result = bytes32_words(x);
/// assert_eq!(result, 4);
/// ```
fn bytes32_words(x: u128) -> u128 {
    (x.saturating_add(31)) / 32
}


struct MyStruct {
    field1: u128,
    field2: u32,
}

fn extract_field_1(my_struct: MyStruct) -> u128 {
    my_struct.field1
}

fn extract_field_2(my_struct: MyStruct) -> u32 {
    my_struct.field2
}

fn my_struct(x: u128, y: u32) -> MyStruct {
    MyStruct { field1: x, field2: y }
}
