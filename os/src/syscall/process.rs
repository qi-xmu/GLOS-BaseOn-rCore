use core::arch::asm;

use crate::loader::*;
use crate::mm::{translated_ref, translated_refmut, translated_str};
use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next,
};
use crate::timer::{get_time_ms, get_time_us, USEC_PER_SEC};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::fs::{open, DiskInodeType, OpenFlags};

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_sched_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_gettimeofday(ts: *mut u64, _tz: usize) -> isize {
    let token = current_user_token();
    let curtime = get_time_us();
    *translated_refmut(token, ts) = (curtime / USEC_PER_SEC) as u64;
    *translated_refmut(token, unsafe { ts.add(1) }) = (curtime % USEC_PER_SEC) as u64;
    0
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

// fork
pub fn sys_clone() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

// 执行应用程序
pub fn sys_execve(path: *const u8, mut args: *const usize) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);

    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe {
            args = args.add(1);
        }
    }
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    let current_path = inner.current_path.as_str();

    /********** 测试开始 *****************/
    // DOING test_all 测试时暂时使用
    if current_path == "/" && path == "test_all" {
        drop(inner); // 释放锁，否则无法继续进行
        task.exec(get_test_binary(), args_vec);
        unsafe {
            asm!("sfence.vma");
            asm!("fence.i"); // 清除TLB
        }
        return 0;
    }
    /**********测试结束******************/

    if let Some(app_inode) = open(
        current_path,
        path.as_str(),
        OpenFlags::RDONLY,
        DiskInodeType::File,
    ) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        let argc = args_vec.len();
        drop(inner);
        task.exec(all_data.as_slice(), args_vec);
        argc as isize
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
/// 如果有子进程正在运行，返回 -2， 如果不存在返回 -1， 否则返回子进程 pid
pub fn sys_wait4(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    // find a child process

    // ---- access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB lock exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child TCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB lock automatically
}
