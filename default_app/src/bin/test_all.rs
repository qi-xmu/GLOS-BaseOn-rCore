#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;

#[macro_use]
#[allow(unused)]
extern crate default_lib;

use alloc::vec::Vec;
use default_lib::*;

fn test() {
    let mut apps = Vec::new();
    apps.push("getcwd\0");
    apps.push("times\0");
    apps.push("gettimeofday\0");
    apps.push("sleep\0");
    apps.push("brk\0");
    // apps.push("clone\0");
    // apps.push("close\0");
    // apps.push("dup2\0");
    // apps.push("dup\0");
    // apps.push("execve\0");
    // apps.push("exit\0");
    // apps.push("fork\0");
    // apps.push("fstat\0");
    // apps.push("getdents\0");
    // apps.push("getpid\0");
    // apps.push("getppid\0");
    // apps.push("mkdir_\0");
    // apps.push("mmap\0");
    // apps.push("munmap\0");
    // apps.push("mount\0");
    // apps.push("openat\0");
    // apps.push("open\0");
    // apps.push("pipe\0");
    // apps.push("read\0");
    // apps.push("umount\0");
    // apps.push("uname\0");
    // apps.push("wait\0");
    // apps.push("waitpid\0");
    // apps.push("write\0");
    // apps.push("yield\0");
    // apps.push("unlink\0");
    // apps.push("chdir\0");
    for app_name in apps {
        let pid = fork();
        if pid == 0 {
            // println!("{} exec end. pid {}", app_name, pid);
            exec(app_name, &[core::ptr::null::<u8>()]);
            return;
        } else {
            let mut exit_code = 0;
            loop {
                let pid = waitpid(pid as usize, &mut exit_code);
                if pid != -2 {
                    break;
                }
            }
            // println!("child proc pid {}", pid);
        }
    }

    shutdown()
}

#[no_mangle]
pub fn main() -> i32 {
    alert!("Start Test All");
    test();
    0
}
