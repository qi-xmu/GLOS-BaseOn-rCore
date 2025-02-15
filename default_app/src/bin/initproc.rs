#![no_std]
#![no_main]

extern crate default_lib;

use default_lib::{exec, fork, wait, yield_};

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        exec("test_all\0", &[core::ptr::null::<u8>()]);
    } else {
        // 不断回收僵尸进程
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                yield_();
                continue;
            }
            // println!(
            //     "[initproc] Released a zombie process, pid={}, exit_code={}",
            //     pid, exit_code,
            // );
        }
    }
    0
}
