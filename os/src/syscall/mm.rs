use crate::task::current_task;

pub fn sys_brk(addr: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    // println!("syscall brk addr = {:x?}, base = {:x?}, top = {:x?}", addr, inner.user_heap_base, inner.user_heap_top);
    let ret = if addr == 0 {
        inner.base_size as isize
    } else if addr >= inner.base_size {
        let addr = addr + inner.base_size;
        addr as isize
    } else {
        -1
    };
    ret
}
