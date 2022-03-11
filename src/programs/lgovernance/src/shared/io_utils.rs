use std::io::Result;

use borsh::BorshSerialize;

#[derive(Debug, BorshSerialize)]
struct Prependable<'a, T: 'a + BorshSerialize> {
    index: u8,
    data: &'a T,
}

pub fn try_to_vec_prepend<T: BorshSerialize>(index: u8, data: &T) -> Result<Vec<u8>> {
    Prependable { index, data }.try_to_vec()
}
