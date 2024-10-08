
use core::dict::{Felt252Dict, Felt252DictTrait};
use core::fmt::{Formatter, Error};

#[derive(Destruct)]
pub struct Stack {
    items: Felt252Dict<u128>,
    size: usize,
}


impl StackDefault of Default<Stack> {
    fn default() -> Stack {
        Stack {
            items: Default::default(),
            size: 0,
        }
    }
}

/// A trait for debug formatting, using the empty format ("{:?}").
pub trait RefDebug<T> {
    fn fmt(ref self: T, ref f: Formatter) -> Result<(), Error>;
}

impl StackDebug of RefDebug<Stack> {
    fn fmt(ref self: Stack, ref f: Formatter) -> Result<(), Error> {
        write!(f, "[").unwrap();
        for i in 0..self.size {
            write!(f, "{}", self.items.get(i.into())).unwrap();
            if i != self.size - 1 {
                write!(f, ", ").unwrap();
            }
        };
        write!(f, "]").unwrap();
        Result::Ok(())
    }
}

#[generate_trait]
impl _stack of StackTrait {
    fn new() -> Stack {
        Stack {
            items: Default::default(),
            size: 0,
        }
    }

    fn len(ref self: Stack) -> usize {
        self.size
    }

    fn push(ref self: Stack, item: u128) {
        self.items.insert(self.len().into(), item);
        self.size += 1;
    }

    fn pop(ref self: Stack) -> u128 {
        let item = self.items.get(self.len().into() - 1);
        self.size -= 1;
        item
    }
}

// #[cfg(test)]
mod tests {
    use super::{Stack, StackTrait, RefDebug};

    #[generate_trait]
    impl StackFromArray of FromArray {
        fn from_array(arr: Array<u128>) -> Stack {
            let mut stack = Default::default();
            for item in arr {
                stack.push(item);
            };
            stack
        }
    }

    fn stack_push_should_add_element(input: Array<u128>, pushed_value: u128) -> ByteArray {
        let mut stack = FromArray::from_array(input);
        stack.push(pushed_value);

        let mut formatter = Default::default();
        RefDebug::fmt(ref stack, ref formatter).unwrap();
        formatter.buffer
    }

    fn stack_pop_should_remove_last_element(input: Array<u128>) -> ByteArray {
        let mut stack = FromArray::from_array(input);
        if stack.len() > 0 {
            let _ = stack.pop();
        }

        let mut formatter = Default::default();
        RefDebug::fmt(ref stack, ref formatter).unwrap();
        formatter.buffer
    }

    fn stack_pop_should_return_last_element(input: Array<u128>) -> u128 {
        let mut stack = FromArray::from_array(input);
        stack.pop()
    }
}
