#![no_std]
#![no_main]

#[macro_use]
extern crate default_lib;

#[no_mangle]
pub fn main() -> i32 {
    alert!("Hello world from user mode program!");
    0
}
