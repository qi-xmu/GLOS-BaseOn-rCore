#![no_std]
// #![feature(linkage)]
// #![feature(panic_info_message)]
#![feature(once_cell)]
extern crate alloc;

mod block_cache;
mod block_dev;
mod fat32_manager;
mod layout;
mod sbi;
mod utils;
mod vfs;

mod fat;

#[macro_use]
mod console;

pub const BLOCK_SZ: usize = 512;

use block_cache::{get_block_cache, get_info_cache, set_start_sec, write_to_dev, CacheMode};
pub use block_dev::BlockDevice;
pub use fat::FAT;
pub use fat32_manager::FAT32Manager;
pub use layout::ShortDirEntry;
pub use layout::*;
pub use vfs::VFile;

// pub use fat::DBR; // test
