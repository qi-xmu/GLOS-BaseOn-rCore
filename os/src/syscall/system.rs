pub fn sys_shutdown() -> isize {
    alert!("[System] The system is shutdown");
    loop {}
}
