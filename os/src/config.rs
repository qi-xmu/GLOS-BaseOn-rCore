// #[allow(unused)]

// 这里是一些全局的变量参数
pub const USER_STACK_SIZE: usize = 4096 * 2; // 8K
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x20_0000;
pub const MEMORY_END: usize = 0x80800000; //8M
pub const PAGE_SIZE: usize = 0x1000; // 4K
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
// pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - PAGE_SIZE;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

pub use crate::board::{CLOCK_FREQ, MMIO};
