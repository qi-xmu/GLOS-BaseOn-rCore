mod dir;
mod inode;
mod pipe;
mod stdio;

mod test; // 测试

use crate::mm::UserBuffer;
use alloc::sync::Arc;

#[derive(Clone)]

// 文件描述符号
pub struct FileDescriptor {
    pub cloexec: bool,
    pub ftype: FileType,
}

impl FileDescriptor {
    pub fn new(flag: bool, ftype: FileType) -> Self {
        Self {
            cloexec: flag,
            ftype: ftype,
        }
    }

    pub fn set_cloexec(&mut self, flag: bool) {
        self.cloexec = flag;
    }

    pub fn get_cloexec(&self) -> bool {
        self.cloexec
    }
}

/// 文件类型
#[derive(Clone)]
pub enum FileType {
    File(Arc<OSInode>),
    Abstr(Arc<dyn File + Send + Sync>),
}

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

pub use dir::{DirEntry, DT_DIR, DT_REG, DT_UNKNOWN};
pub use inode::{list_apps, open, DiskInodeType, OSInode, OpenFlags, ROOT_VFILE};
pub use pipe::{make_pipe, Pipe};
pub use stdio::{Stdin, Stdout};
