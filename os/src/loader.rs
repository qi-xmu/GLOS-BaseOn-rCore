use core::slice::from_raw_parts;

pub fn get_initproc_binary() -> &'static [u8] {
    extern "C" {
        fn sinitproc();
        fn einitproc();
    }
    unsafe {
        from_raw_parts(
            sinitproc as *const u8,
            einitproc as usize - sinitproc as usize,
        )
    }
}

pub fn get_test_binary() -> &'static [u8] {
    extern "C" {
        fn stest();
        fn etest();
    }
    unsafe { from_raw_parts(stest as *const u8, etest as usize - stest as usize) }
}

pub fn get_hello_binary() -> &'static [u8] {
    extern "C" {
        fn shello_world();
        fn ehello_world();
    }
    unsafe {
        from_raw_parts(
            shello_world as *const u8,
            ehello_world as usize - shello_world as usize,
        )
    }
}
