#![no_std]
#![no_main]
#![allow(unused)]
#![allow(clippy::println_empty_string)]

extern crate alloc;

#[macro_use]
extern crate default_lib;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;
const LINE_START: &str = ">> ";

use alloc::string::String;
use alloc::vec::Vec;
use default_lib::*;

#[derive(Debug)]
struct ProcessArguments {
    input: String,
    output: String,
    args_copy: Vec<String>,
    args_addr: Vec<*const u8>,
}

impl ProcessArguments {
    #[allow(unused)]
    pub fn new(command: &str) -> Self {
        let args: Vec<_> = command.split(' ').collect();
        let mut args_copy: Vec<String> = args
            .iter()
            .filter(|&arg| !arg.is_empty())
            .map(|&arg| {
                let mut string = String::new();
                string.push_str(arg);
                string.push('\0');
                string
            })
            .collect();

        // redirect input
        let mut input = String::new();
        if let Some((idx, _)) = args_copy
            .iter()
            .enumerate()
            .find(|(_, arg)| arg.as_str() == "<\0")
        {
            input = args_copy[idx + 1].clone();
            args_copy.drain(idx..=idx + 1);
        }

        // redirect output
        let mut output = String::new();
        if let Some((idx, _)) = args_copy
            .iter()
            .enumerate()
            .find(|(_, arg)| arg.as_str() == ">\0")
        {
            output = args_copy[idx + 1].clone();
            args_copy.drain(idx..=idx + 1);
        }

        let mut args_addr: Vec<*const u8> = args_copy.iter().map(|arg| arg.as_ptr()).collect();
        args_addr.push(core::ptr::null::<u8>());

        Self {
            input,
            output,
            args_copy,
            args_addr,
        }
    }
}

fn test() -> ! {
    let mut preliminary_apps = Vec::new();
    preliminary_apps.push("times\0");
    preliminary_apps.push("gettimeofday\0");
    preliminary_apps.push("sleep\0");
    preliminary_apps.push("brk\0");
    preliminary_apps.push("clone\0");
    preliminary_apps.push("close\0");
    preliminary_apps.push("dup2\0");
    preliminary_apps.push("dup\0");
    preliminary_apps.push("execve\0");
    preliminary_apps.push("exit\0");
    preliminary_apps.push("fork\0");
    preliminary_apps.push("fstat\0");
    preliminary_apps.push("getcwd\0");
    preliminary_apps.push("getdents\0");
    preliminary_apps.push("getpid\0");
    preliminary_apps.push("getppid\0");
    preliminary_apps.push("mkdir_\0");
    preliminary_apps.push("mmap\0");
    preliminary_apps.push("munmap\0");
    preliminary_apps.push("mount\0");
    preliminary_apps.push("openat\0");
    preliminary_apps.push("open\0");
    preliminary_apps.push("pipe\0");
    preliminary_apps.push("read\0");
    preliminary_apps.push("umount\0");
    preliminary_apps.push("uname\0");
    preliminary_apps.push("wait\0");
    preliminary_apps.push("waitpid\0");
    preliminary_apps.push("write\0");
    preliminary_apps.push("yield\0");
    preliminary_apps.push("unlink\0");
    preliminary_apps.push("chdir\0");

    for app_name in preliminary_apps {
        let pid = fork();
        if pid == 0 {
            exec(app_name, &[core::ptr::null::<u8>()]);
        } else {
            let mut exit_code = 0;
            waitpid(pid as usize, &mut exit_code);
        }
    }

    shutdown()
}

#[no_mangle]
pub fn main() -> i32 {
    println!("Start Test All");
    test();
}
