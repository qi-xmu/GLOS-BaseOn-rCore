use k210_soc::sleep::usleep;

use crate::mm::{translated_ref, translated_refmut};
use crate::task::suspend_current_and_run_next;
use crate::timer::*;
use crate::{sbi::shutdown, task::current_user_token};

pub fn sys_shutdown() -> ! {
    shutdown();
}

pub fn sys_times(time: *mut usize) -> isize {
    let token = current_user_token();
    let sec = get_time_us();
    *translated_refmut(token, time) = sec;
    *translated_refmut(token, unsafe { time.add(1) }) = sec;
    *translated_refmut(token, unsafe { time.add(2) }) = sec;
    *translated_refmut(token, unsafe { time.add(3) }) = sec;
    0
}

pub fn sys_nanosleep(timespec: *mut u64) -> isize {
    let token = current_user_token();
    let sec = *translated_ref(token, timespec);
    let usec = *translated_ref(token, unsafe { timespec.add(1) });
    drop(token);

    let total_usec = sec as usize * USEC_PER_SEC + usec as usize;

    let start_time = get_time_us();
    while get_time_us() - start_time < total_usec {
        suspend_current_and_run_next();
    }
    0
}
