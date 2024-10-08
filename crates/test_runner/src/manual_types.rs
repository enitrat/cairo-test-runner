use cairo_lang_runner::short_string::{as_cairo_short_string, as_cairo_short_string_ex};
use itertools::Itertools;
use num_traits::cast::ToPrimitive;
use starknet_types_core::felt::Felt;
use cairo_lang_utils::byte_array::{BYTES_IN_WORD, BYTE_ARRAY_MAGIC};

#[derive(Debug, PartialEq, Eq)]
pub struct MyStruct {
    pub field_0: u128,
    pub field_1: u32,
}

impl TryFrom<Vec<Felt>> for MyStruct {
    type Error = String;
    fn try_from(vec: Vec<Felt>) -> Result<Self, String> {
        Ok(Self {
            field_0: vec[0].try_into().unwrap(),
            field_1: vec[1].try_into().unwrap(),
        })    }
}


#[derive(Debug, PartialEq, Eq)]
pub struct Stack(Vec<u128>);

impl Stack {
    pub fn from_array(values: Vec<u128>) -> Stack {
        Stack(values)
    }
}

impl TryFrom<Vec<Felt>> for Stack {
    type Error = String;

    fn try_from(values: Vec<Felt>) -> Result<Self, Self::Error> {
        Ok(Stack(from_felt_byte_array(values).map_err(|e| e.to_string())?))
    }
}

pub fn from_felt_byte_array(felt_bytes: Vec<Felt>) -> Result<Vec<u128>, ParseIntError> {
    let string = format_for_debug(felt_bytes.into_iter());
    // string in form [1,2,3]
    // remove brackets and split by commas
    let values_str = string.trim_matches(|c| c == '[' || c == ']');
    if values_str.is_empty() {
        return Ok(vec![]);
    }
    println!("values_str: {:?}", values_str);
    let values: Result<Vec<u128>, _> = values_str
        .split(',')
        .map(|s| s.trim().parse::<u128>())
        .collect();
    println!("values: {:?}", values);
    values
}

#[derive(Debug, PartialEq, Eq)]
pub struct U8(pub u8);
#[derive(Debug, PartialEq, Eq)]
pub struct U16(pub u16);
#[derive(Debug, PartialEq, Eq)]
pub struct U32(pub u32);
#[derive(Debug, PartialEq, Eq)]
pub struct U64(pub u64);
#[derive(Debug, PartialEq, Eq)]
pub struct U128(pub u128);

use std::{num::ParseIntError, ops::Deref, vec::IntoIter};

macro_rules! impl_try_from_felt_vec_and_deref {
    ($($t:ty, $inner:ty),+) => {
        $(
            impl TryFrom<Vec<Felt>> for $t {
                type Error = String;

                fn try_from(values: Vec<Felt>) -> Result<Self, Self::Error> {
                    match values.len() {
                        1 => Ok(Self(values[0].try_into().map_err(|e| format!("Conversion error: {}", e))?)),
                        _ => Err("Invalid number of values".to_string()),
                    }
                }
            }

            impl Deref for $t {
                type Target = $inner;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        )+
    };
}

impl_try_from_felt_vec_and_deref!(U8, u8, U16, u16, U32, u32, U64, u64, U128, u128);


/// Formats the given felts as a debug string.
fn format_for_debug(mut felts: IntoIter<Felt>) -> String {
    let mut items = Vec::new();
    while let Some(item) = format_next_item(&mut felts) {
        items.push(item);
    }
    if let [item] = &items[..] {
        if item.is_string {
            return item.item.clone();
        }
    }
    items
        .into_iter()
        .map(|item| {
            if item.is_string {
                format!("{}\n", item.item)
            } else {
                println!("item.item: {:?}", item.item);
                format!("[DEBUG]\t{}\n", item.item)
            }
        })
        .join("")
}

/// A formatted string representation of anything formattable (e.g. ByteArray, felt, short-string).
pub struct FormattedItem {
    /// The formatted string representing the item.
    item: String,
    /// Whether the item is a string.
    is_string: bool,
}
impl FormattedItem {
    /// Returns the formatted item as is.
    pub fn get(self) -> String {
        self.item
    }
    /// Wraps the formatted item with quote, if it's a string. Otherwise returns it as is.
    pub fn quote_if_string(self) -> String {
        if self.is_string { format!("\"{}\"", self.item) } else { self.item }
    }
}

/// Formats a string or a short string / `Felt`. Returns the formatted string and a boolean
/// indicating whether it's a string. If can't format the item, returns None.
pub fn format_next_item<T>(values: &mut T) -> Option<FormattedItem>
where
    T: Iterator<Item = Felt> + Clone,
{
    let first_felt = values.next()?;

    if first_felt == Felt::from_hex(BYTE_ARRAY_MAGIC).unwrap() {
        if let Some(string) = try_format_string(values) {
            return Some(FormattedItem { item: string, is_string: true });
        }
    }
    Some(FormattedItem { item: format_short_string(&first_felt), is_string: false })
}

/// Formats a `Felt`, as a short string if possible.
fn format_short_string(value: &Felt) -> String {
    let hex_value = value.to_biguint();
    match as_cairo_short_string(value) {
        Some(as_string) => format!("{hex_value:#x} ('{as_string}')"),
        None => format!("{hex_value:#x}"),
    }
}

/// Tries to format a string, represented as a sequence of `Felt`s.
/// If the sequence is not a valid serialization of a ByteArray, returns None and doesn't change the
/// given iterator (`values`).
fn try_format_string<T>(values: &mut T) -> Option<String>
where
    T: Iterator<Item = Felt> + Clone,
{
    // Clone the iterator and work with the clone. If the extraction of the string is successful,
    // change the original iterator to the one we worked with. If not, continue with the
    // original iterator at the original point.
    let mut cloned_values_iter = values.clone();

    let num_full_words = cloned_values_iter.next()?.to_usize()?;
    let full_words = cloned_values_iter.by_ref().take(num_full_words).collect_vec();
    let pending_word = cloned_values_iter.next()?;
    let pending_word_len = cloned_values_iter.next()?.to_usize()?;

    let full_words_string = full_words
        .into_iter()
        .map(|word| as_cairo_short_string_ex(&word, BYTES_IN_WORD))
        .collect::<Option<Vec<String>>>()?
        .join("");
    let pending_word_string = as_cairo_short_string_ex(&pending_word, pending_word_len)?;

    // Extraction was successful, change the original iterator to the one we worked with.
    *values = cloned_values_iter;

    Some(format!("{full_words_string}{pending_word_string}"))
}
