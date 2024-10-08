mod utils;
mod stack;

// fn main() -> Felt252Dict<u64>{
//     let mut dict = Default::default();

//     dict.insert(1, 2);
//     dict.insert(2, 3);
//     dict.insert(3, 4);

//     return dict;
// }

fn main() -> Box<u64>{
    let mut x = 3;
    return BoxTrait::new(x);
}
