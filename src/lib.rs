#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]
#[cfg(test)]
extern crate quickcheck;

extern crate hashtable_sys;

pub mod error;
pub mod hashtable;
