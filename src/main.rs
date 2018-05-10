#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]
#[cfg(test)]
extern crate quickcheck;

extern crate hashtable_sys;

mod error;
mod hashtable;

fn main() {
    let mut hashtable = hashtable::HashTable::<u8, u8>::new().unwrap();
    hashtable.set(3u8, 8u8).unwrap();
    println!("{:?}", hashtable.get(&3u8));
}
