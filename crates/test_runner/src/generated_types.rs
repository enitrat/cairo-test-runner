use starknet_types_core::felt::Felt;
#[derive(Debug, PartialEq, Eq)]
pub struct MyStruct {
    pub field_0: u128,
    pub field_1: u32,
}

impl From<Vec<Felt>> for MyStruct {
    fn from(vec: Vec<Felt>) -> Self {
        Self {
            field_0: vec[0].try_into().unwrap(),
            field_1: vec[1].try_into().unwrap(),
        }
    }
}

