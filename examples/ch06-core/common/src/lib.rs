#![no_std]

mod memory;
mod string;

pub use memory::{memcpy, memset};
pub use string::{strcmp, strcpy};
