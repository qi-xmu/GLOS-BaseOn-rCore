//! File and filesystem-related syscalls
use crate::mm::{translated_byte_buffer, translated_str};
use crate::sbi::console_getchar;
use crate::task::{current_task, current_user_token, suspend_current_and_run_next};

use crate::fs::{open, DiskInodeType, FileDescriptor, FileType, OpenFlags};

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

pub fn sys_openat(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    let open_flags = OpenFlags::from_bits(flags).unwrap();
    let mut inner = task.inner_exclusive_access();
    if let Some(inode) = open(
        inner.get_work_path().as_str(),
        path.as_str(),
        open_flags,
        DiskInodeType::File,
    ) {
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(FileDescriptor::new(
            open_flags.contains(OpenFlags::CLOEXEC),
            FileType::File(inode),
        ));
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "Only support len = 1 in sys_read!");
            let mut c: usize;
            loop {
                c = console_getchar();
                if c == 0 {
                    suspend_current_and_run_next();
                    continue;
                } else {
                    break;
                }
            }
            let ch = c as u8;
            let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
            unsafe {
                buffers[0].as_mut_ptr().write_volatile(ch);
            }
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}
