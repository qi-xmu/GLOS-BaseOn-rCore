use crate::utils::clone_into_array;
use crate::FAT;

use super::{
    fat32_manager::FAT32Manager, get_block_cache, get_info_cache, BlockDevice, CacheMode, BLOCK_SZ,
};
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

// fat info FSI_LeadSig 固定值0x41615252
pub const LEAD_SIGNATURE: u32 = 0x41615252;
// fat info FSI_StructSig 固定值0x61417272
pub const STRUCT_SIGNATURE: u32 = 0x61417272;
// FSI_Free_Count
// 此值的含义是当前分区free cluster的个数，如果此值为0Xffffffff,那么则说明free cluster的个数是未知的
pub const FREE_CLUSTER: u32 = 0x00000000;
// 对于FAT32而言，代表文件结束的FAT表项值为0x0FFFFFFF。
// 0x0FFFFFF8;FAT表起始固定标识
pub const END_CLUSTER: u32 = 0x0FFFFFF8;
// 如果某个簇存在坏扇区，则整个簇会用0xFFFFFF7标记为坏簇，这个坏簇标记就记录在它所对应的FAT表项中
pub const BAD_CLUSTER: u32 = 0x0FFFFFF7;
// 每一个sector 的 entry
pub const FATENTRY_PER_SEC: u32 = BLOCK_SZ as u32 / 4; // 128

// 0000_0000 读写
#[allow(unused)]
pub const ATTRIBUTE_READ_ONLY: u8 = 0x01; // 0000_0001 只读
pub const ATTRIBUTE_HIDDEN: u8 = 0x02; // 0000_0010 隐藏
pub const ATTRIBUTE_SYSTEM: u8 = 0x04; // 0000_0100 系统
pub const ATTRIBUTE_VOLUME_ID: u8 = 0x08; // 0000_1000 卷标
pub const ATTRIBUTE_DIRECTORY: u8 = 0x10; // 0001_0000 目录
pub const ATTRIBUTE_ARCHIVE: u8 = 0x20; // 0010_0000 归档
pub const ATTRIBUTE_LFN: u8 = 0x0F; // 0000_1111 长目录标志

#[allow(unused)]
pub const DIRENT_SZ: usize = 32;
pub const SHORT_NAME_LEN: usize = 8;
pub const SHORT_EXT_LEN: usize = 3;
pub const LONG_NAME_LEN: usize = 13;

pub const ALL_UPPER_CASE: u8 = 0x00;
pub const ALL_LOWER_CASE: u8 = 0x08;

type DataBlock = [u8; BLOCK_SZ]; //数据块类型

#[repr(packed)]
#[derive(Clone, Copy, Debug)]

// DBR DOS Boot Recorder
pub struct FatBS {
    pub unused: [u8; 11],           // 0x00 0x03 跳转指令 + OEM
    pub bytes_per_sector: u16,      // 0x0b 扇区字节数
    pub sectors_per_cluster: u8,    // 0x0d 每簇扇区数
    pub reserved_sector_count: u16, // 0x0e 保留扇区数
    pub table_count: u8,            // 0x10 文件分配表个数， 一般为 2
    pub root_entry_count: u16,      // 0x11 最大根目录条目个数 FAT32必须等于0
    pub total_sectors_16: u16,      // 0x13 总扇区个数
    pub media_type: u8,             // 0x15 介质介绍
    pub table_size_16: u16,         // 0x16 文件分配表的扇区 FAT16
    pub sectors_per_track: u16,     // 0x18 每个磁道的扇区
    pub head_side_count: u16,       // 0x1a 磁头数量
    pub hidden_sector_count: u32,   // 0x1c 隐藏扇区数量
    pub total_sectors_32: u32,      // 0x20 总扇区数量
}

impl FatBS {
    // 这个地方是生成 BIOSParamterBlock
    // pub fn init_boot_sector(block_device: Arc<dyn BlockDevice>) {
    //     // 文件系统的起始扇区为0号扇区。
    //     let cache = get_info_cache(0, block_device, CacheMode::WRITE);
    //     let mut guard = cache.write();
    //     // 防止冲突的写入，偏移为 0
    //     guard.modify(0, |fat_bs: &mut FatBS| {
    //         *fat_bs = FatBS {
    //             unused: [0u8; 11],
    //             bytes_per_sector: BLOCK_SZ as u16,
    //             sectors_per_cluster: 1,
    //             reserved_sector_count: 2,
    //             table_count: 2,
    //             root_entry_count: 0,
    //             total_sectors_16: 0,
    //             media_type: 0,
    //             table_size_16: 0,
    //             sectors_per_track: 0,
    //             head_side_count: 0,
    //             hidden_sector_count: 0,
    //             total_sectors_32: TOTAL_SECTORS as u32,
    //         }
    //     });

    //     drop(guard);
    // }

    pub fn total_sectors(&self) -> u32 {
        if self.total_sectors_16 == 0 {
            self.total_sectors_32
        } else {
            self.total_sectors_16 as u32
        }
    }

    /// 第一个FAT表所在的扇区
    pub fn first_fat_sector(&self) -> u32 {
        // println!(
        //     "\x1b[31m[fat] reser_fat-sec {} \x1b[0m",
        //     self.reserved_sector_count as u32
        // );
        self.reserved_sector_count as u32
    }
}

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
#[allow(unused)]
pub struct FatExtBS {
    pub table_size_32: u32,    // 0x24 每个分配表的扇区 FAT32
    pub extended_flags: u16,   // 0x28 Flag FAT32
    pub fat_version: u16,      // 0x2a 版本号 FAT32
    pub root_clusters: u32,    // 0x2c 根目录启动簇 FAT32
    pub fat_info: u16,         // 0x30 fat info 扇区, 一般为1 FAT32
    pub backup_bs_sector: u16, // 0x32 启动扇区备份 FAT32
    pub reserved_0: [u8; 12],  // 0x34 保留 FAT32
    pub drive_number: u8,      // 0x40 BIOS设备代号 FAT32
    pub reserved_1: u8,        // 0x41 未使用 FAT32
    pub boot_signature: u8,    // 0x42 标记 0x28 or 0x29 FAT32
}

impl FatExtBS {
    // 这个地方是生成 FATEXTBS
    // pub fn init_ext_bs(block_device: Arc<dyn BlockDevice>) {
    //     let cache = get_info_cache(0, block_device, CacheMode::WRITE);
    //     let mut guard = cache.write();
    //     guard.modify(36, |fat_ext_bs: &mut FatExtBS| {
    //         *fat_ext_bs = FatExtBS {
    //             table_size_32: TABLE_SIZE as u32,
    //             extended_flags: 0,
    //             fat_version: 0,
    //             root_clusters: 2,
    //             fat_info: 1,
    //             backup_bs_sector: 0,
    //             reserved_0: [0u8; 12],
    //             drive_number: 0x80,
    //             reserved_1: 0,
    //             boot_signature: 0,
    //         };
    //     });
    //     drop(guard);
    // }

    /// FAT占用的扇区数
    pub fn fat_size(&self) -> u32 {
        self.table_size_32
    }

    /// FSINFO（文件系统信息扇区）扇区号是1，该扇区为操作系统提供关于空簇总数及下一可用簇的信息。
    pub fn fat_info_sec(&self) -> u32 {
        self.fat_info as u32
    }

    #[allow(unused)]
    pub fn root_clusters(&self) -> u32 {
        self.root_clusters
    }
}

/// 该结构体不对Buffer作结构映射，仅保留位置信息
/// 但是为其中信息的获取和修改提供了接口
pub struct FSInfo {
    sector_num: u32,
}

impl FSInfo {
    pub fn new(sector_num: u32) -> Self {
        Self { sector_num } // sector_num = 1
    }

    // 这个地方是生成 FAT,
    // /// 初始化FSInfo，并写入文件系统镜像
    // pub fn init_fsinfo(&self, block_device: Arc<dyn BlockDevice>) {
    //     let cache = get_info_cache(self.sector_num as usize, block_device, CacheMode::WRITE);
    //     let mut guard = cache.write();
    //     // FSI_LeadSig = 4
    //     guard.modify(0, |leadsig: &mut u32| {
    //         *leadsig = bytes_order_u32(LEAD_SIGNATURE);
    //     });
    //     // FSI_LeadSig + FSI_Reservedl = 484
    //     guard.modify(484, |sec_sig: &mut u32| {
    //         *sec_sig = bytes_order_u32(SECOND_SIGNATURE);
    //     });
    //     drop(guard);
    // }

    // 检查 lead signature
    fn check_lead_signature(&self, block_device: Arc<dyn BlockDevice>) -> bool {
        get_info_cache(self.sector_num as usize, block_device, CacheMode::READ)
            .read()
            .read(0, |&lead_sig: &u32| lead_sig == LEAD_SIGNATURE)
    }

    fn check_struct_signature(&self, block_device: Arc<dyn BlockDevice>) -> bool {
        get_info_cache(self.sector_num as usize, block_device, CacheMode::READ)
            .read()
            .read(484, |&sec_sig: &u32| sec_sig == STRUCT_SIGNATURE)
    }

    /// 对签名进行校验
    pub fn check_signature(&self, block_device: Arc<dyn BlockDevice>) -> bool {
        return self.check_lead_signature(block_device.clone())
            && self.check_struct_signature(block_device.clone());
    }

    /// 读取空闲簇数
    pub fn read_free_clusters(&self, block_device: Arc<dyn BlockDevice>) -> u32 {
        get_info_cache(self.sector_num as usize, block_device, CacheMode::READ)
            .read()
            .read(488, |&free_cluster_count: &u32| free_cluster_count)
    }

    /// 写空闲块数
    pub fn write_free_clusters(&self, free_clusters: u32, block_device: Arc<dyn BlockDevice>) {
        get_info_cache(self.sector_num as usize, block_device, CacheMode::WRITE)
            .write()
            .modify(488, |free_cluster_count: &mut u32| {
                *free_cluster_count = free_clusters;
            });
    }

    /// 读起始空闲块
    pub fn first_free_cluster(&self, block_device: Arc<dyn BlockDevice>) -> u32 {
        get_info_cache(self.sector_num as usize, block_device, CacheMode::READ)
            .read()
            .read(492, |&start_cluster: &u32| start_cluster)
    }

    /// 写起始空闲块
    pub fn write_first_free_cluster(&self, start_cluster: u32, block_device: Arc<dyn BlockDevice>) {
        //println!("sector_num = {}, start_c = {}", self.sector_num, start_cluster);
        get_info_cache(self.sector_num as usize, block_device, CacheMode::WRITE)
            .write()
            .modify(492, |start_clu: &mut u32| {
                *start_clu = start_cluster;
            });
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(packed)]
#[allow(unused)]
// 短文件目录项的具体定义
pub struct ShortDirEntry {
    // 删除时第0位为0xE5，未使用时为0x00. 有多余可以用0x20填充
    pub name: [u8; 8],          // 文件名
    pub extension: [u8; 3],     // 扩展名
    pub attribute: u8,          // 属性字节 可以用于判断是目录还是文件
    pub winnt_reserved: u8,     // 系统保留
    pub creation_tenths: u8,    // 创建时间 精确到0.1s
    pub creation_time: u16,     // 文件创建时间
    pub creation_date: u16,     // 文件创建日期
    pub last_acc_date: u16,     // 文件最后访问时间
    pub cluster_high: u16,      // 文件起始簇号的高16位
    pub modification_time: u16, // 文件最近修改时间
    pub modification_date: u16, // 文件最近修改日期
    pub cluster_low: u16,       // 文件起始簇号的低16位
    pub size: u32,              // 文件长度
}

impl ShortDirEntry {
    pub fn empty() -> Self {
        Self {
            name: [0; 8], // 删除时第0位为0xE5，未使用时为0x00. 有多余可以用0x20填充
            extension: [0; 3],
            attribute: 0, //可以用于判断是目录还是文件
            winnt_reserved: 0,
            creation_tenths: 0, //精确到0.1s
            creation_time: 0,
            creation_date: 0,
            last_acc_date: 0,
            cluster_high: 0,
            modification_time: 0,
            modification_date: 0,
            cluster_low: 0,
            size: 0,
        }
    }

    /// 创建文件时调用
    /// 新建时不必分配块。写时检测初始簇是否为0，为0则需要分配
    pub fn new(name_: &[u8], extension_: &[u8], attribute: u8) -> Self {
        let name: [u8; 8] = clone_into_array(&name_[0..8]);
        let extension: [u8; 3] = clone_into_array(&extension_[0..3]);
        Self {
            name,
            extension,
            attribute,
            winnt_reserved: 0,
            creation_tenths: 0,
            creation_time: 0,
            creation_date: 0x529c,
            last_acc_date: 0,
            cluster_high: 0,
            modification_time: 0,
            modification_date: 0,
            cluster_low: 0,
            size: 0,
        }
    }

    /// 初始化短文件目录项
    pub fn initialize(&mut self, name_: &[u8], extension_: &[u8], attribute: u8) {
        let name: [u8; 8] = clone_into_array(&name_[0..8]);
        let extension: [u8; 3] = clone_into_array(&extension_[0..3]);
        *self = Self {
            name,
            extension,
            attribute,
            winnt_reserved: 0,
            creation_tenths: 0,
            creation_time: 0,
            creation_date: 0x529c,
            last_acc_date: 0,
            cluster_high: 0,
            modification_time: 0,
            modification_date: 0,
            cluster_low: 0,
            size: 0,
        };
    }

    /// 返回目前使用的簇的数量
    pub fn data_clusters(&self, bytes_per_cluster: u32) -> u32 {
        // size为0的时候就是0
        (self.size + bytes_per_cluster - 1) / bytes_per_cluster
    }

    pub fn is_dir(&self) -> bool {
        if 0 != (self.attribute & ATTRIBUTE_DIRECTORY) {
            true
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.name[0] == 0xE5 {
            false
        } else {
            true
        }
    }

    pub fn is_deleted(&self) -> bool {
        if self.name[0] == 0xE5 {
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        if self.name[0] == 0x00 {
            true
        } else {
            false
        }
    }

    pub fn is_file(&self) -> bool {
        if 0 != (self.attribute & ATTRIBUTE_DIRECTORY) {
            false
        } else {
            true
        }
    }

    pub fn is_long(&self) -> bool {
        if self.attribute == ATTRIBUTE_LFN {
            true
        } else {
            false
        }
    }

    pub fn attribute(&self) -> u8 {
        self.attribute
    }

    pub fn get_creation_time(&self) -> (u32, u32, u32, u32, u32, u32, u64) {
        // year-month-day-Hour-min-sec-long_sec
        let year: u32 = ((self.creation_date & 0xFE00) >> 9) as u32 + 1980;
        let month: u32 = ((self.creation_date & 0x01E0) >> 5) as u32;
        let day: u32 = (self.creation_date & 0x001F) as u32;
        let hour: u32 = ((self.creation_time & 0xF800) >> 11) as u32;
        let min: u32 = ((self.creation_time & 0x07E0) >> 5) as u32;
        let sec: u32 = ((self.creation_time & 0x001F) << 1) as u32; // 秒数需要*2
        let long_sec: u64 =
            ((((year - 1970) * 365 + month * 30 + day) * 24 + hour) * 3600 + min * 60 + sec) as u64;
        (year, month, day, hour, min, sec, long_sec)
    }

    pub fn get_modification_time(&self) -> (u32, u32, u32, u32, u32, u32, u64) {
        // year-month-day-Hour-min-sec
        let year: u32 = ((self.modification_date & 0xFE00) >> 9) as u32 + 1980;
        let month: u32 = ((self.modification_date & 0x01E0) >> 5) as u32;
        let day: u32 = (self.modification_date & 0x001F) as u32;
        let hour: u32 = ((self.modification_time & 0xF800) >> 11) as u32;
        let min: u32 = ((self.modification_time & 0x07E0) >> 5) as u32;
        let sec: u32 = ((self.modification_time & 0x001F) << 1) as u32; // 秒数需要*2
        let long_sec: u64 =
            ((((year - 1970) * 365 + month * 30 + day) * 24 + hour) * 3600 + min * 60 + sec) as u64;
        (year, month, day, hour, min, sec, long_sec)
    }

    pub fn get_accessed_time(&self) -> (u32, u32, u32, u32, u32, u32, u64) {
        // year-month-day-Hour-min-sec
        let year: u32 = ((self.last_acc_date & 0xFE00) >> 9) as u32 + 1980;
        let month: u32 = ((self.last_acc_date & 0x01E0) >> 5) as u32;
        let day: u32 = (self.last_acc_date & 0x001F) as u32;
        let hour: u32 = 0;
        let min: u32 = 0;
        let sec: u32 = 0; // 没有相关信息，默认0
        let long_sec: u64 =
            ((((year - 1970) * 365 + month * 30 + day) * 24 + hour) * 3600 + min * 60 + sec) as u64;
        (year, month, day, hour, min, sec, long_sec)
    }

    /// 获取文件起始簇号
    pub fn first_cluster(&self) -> u32 {
        ((self.cluster_high as u32) << 16) + (self.cluster_low as u32)
    }

    /// 获取短文件名
    pub fn get_name_uppercase(&self) -> String {
        let mut name: String = String::new();
        for i in 0..8 {
            // 记录文件名
            if self.name[i] == 0x20 {
                break;
            } else {
                name.push(self.name[i] as char);
            }
        }
        for i in 0..3 {
            // 记录扩展名
            if self.extension[i] == 0x20 {
                break;
            } else {
                if i == 0 {
                    name.push('.');
                }
                name.push(self.extension[i] as char);
            }
        }
        name
    }

    pub fn get_name_lowercase(&self) -> String {
        // 获取名字长度
        let name_len = (0usize..SHORT_NAME_LEN)
            .find(|i| self.name[*i] == 0x20u8)
            .unwrap_or(SHORT_NAME_LEN);
        // 名字
        let name = core::str::from_utf8(&self.name[..name_len]).unwrap();

        // 获取扩展的长度
        let ext_len = (0usize..SHORT_EXT_LEN)
            .find(|i| self.extension[*i] == 0x20)
            .unwrap_or(SHORT_EXT_LEN);
        // 扩展名
        let extension = core::str::from_utf8(&self.extension[0..ext_len]).unwrap();
        // 组合
        if ext_len == 0 {
            String::from(name).to_ascii_lowercase()
        } else {
            format!(
                "{}.{}",
                name.to_ascii_lowercase(),
                extension.to_ascii_lowercase()
            )
        }
    }

    /// 计算校验和
    pub fn checksum(&self) -> u8 {
        let mut name_buff: [u8; 11] = [0u8; 11];
        let mut sum: u8 = 0;
        for i in 0..8 {
            name_buff[i] = self.name[i];
        }
        for i in 0..3 {
            name_buff[i + 8] = self.extension[i];
        }
        for i in 0..11 {
            if (sum & 1) != 0 {
                sum = 0x80 + (sum >> 1) + name_buff[i];
            } else {
                sum = (sum >> 1) + name_buff[i];
            }
        }
        sum
    }

    /// 设置当前文件的大小
    /// 簇的分配和回收实际要对FAT表操作
    pub fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    pub fn set_case(&mut self, case: u8) {
        self.winnt_reserved = case;
    }

    /// 设置文件起始簇
    pub fn set_first_cluster(&mut self, cluster: u32) {
        self.cluster_high = ((cluster & 0xFFFF0000) >> 16) as u16; // 高16位
        self.cluster_low = (cluster & 0x0000FFFF) as u16; // 底16位
    }

    /// 清空文件，删除时使用
    pub fn clear(&mut self) {
        self.size = 0;
        //self.name[0] = 0xE5;
        self.set_first_cluster(0);
    }

    pub fn delete(&mut self) {
        self.size = 0;
        self.name[0] = 0xE5;
        self.set_first_cluster(0);
    }

    /// 获取文件偏移量所在的簇、扇区和偏移
    pub fn get_pos(
        &self,
        offset: usize,
        manager: &Arc<RwLock<FAT32Manager>>,
        fat: &Arc<RwLock<FAT>>,
        block_device: &Arc<dyn BlockDevice>,
    ) -> (u32, usize, usize) {
        let manager_reader = manager.read();
        let fat_reader = fat.read();
        let bytes_per_sector = manager_reader.bytes_per_sector() as usize;
        let bytes_per_cluster = manager_reader.bytes_per_cluster() as usize;
        let cluster_index = manager_reader.cluster_of_offset(offset);
        let current_cluster = fat_reader.get_cluster_at(
            self.first_cluster(),
            cluster_index,
            Arc::clone(block_device),
        );
        let current_sector = manager_reader.first_sector_of_cluster(current_cluster)
            + (offset - cluster_index as usize * bytes_per_cluster) / bytes_per_sector;
        (current_cluster, current_sector, offset % bytes_per_sector)
    }

    /// 以偏移量读取文件，这里会对fat和manager加读锁
    pub fn read_at(
        &self,
        offset: usize,
        buf: &mut [u8],
        manager: &Arc<RwLock<FAT32Manager>>,
        fat: &Arc<RwLock<FAT>>,
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        // 获取共享锁
        let manager_reader = manager.read();
        let fat_reader = fat.read();
        let bytes_per_sector = manager_reader.bytes_per_sector() as usize;
        let bytes_per_cluster = manager_reader.bytes_per_cluster() as usize;
        let mut current_off = offset;
        let end: usize;
        if self.is_dir() {
            let size = bytes_per_cluster
                * fat_reader.count_cluster_num(self.first_cluster() as u32, block_device.clone())
                    as usize;
            end = offset + buf.len().min(size); // DEBUG:约束上界
        } else {
            end = (offset + buf.len()).min(self.size as usize);
        }
        if current_off >= end {
            return 0;
        }
        let (curr_clu, curr_sec, _) = self.get_pos(offset, manager, fat, block_device);
        if curr_clu >= END_CLUSTER {
            return 0;
        };
        let mut current_cluster = curr_clu;
        let mut current_sector = curr_sec;

        let mut read_size = 0usize;
        loop {
            // 将偏移量向上对齐扇区大小（一般是512
            let mut end_current_block = (current_off / bytes_per_sector + 1) * bytes_per_sector;
            end_current_block = end_current_block.min(end);
            // 读
            let block_read_size = end_current_block - current_off;
            let dst = &mut buf[read_size..read_size + block_read_size];
            if self.is_dir() {
                get_info_cache(
                    // 目录项通过Infocache访问
                    current_sector,
                    Arc::clone(block_device),
                    CacheMode::READ,
                )
                .read()
                .read(0, |data_block: &DataBlock| {
                    let src = &data_block
                        [current_off % BLOCK_SZ..current_off % BLOCK_SZ + block_read_size];
                    dst.copy_from_slice(src);
                });
            } else {
                get_block_cache(current_sector, Arc::clone(block_device), CacheMode::READ)
                    .read()
                    .read(0, |data_block: &DataBlock| {
                        let src = &data_block
                            [current_off % BLOCK_SZ..current_off % BLOCK_SZ + block_read_size];
                        dst.copy_from_slice(src);
                    });
            }
            // 更新读取长度
            read_size += block_read_size;
            if end_current_block == end {
                break;
            }
            // 更新索引参数
            current_off = end_current_block;
            if current_off % bytes_per_cluster == 0 {
                // 读完一个簇
                current_cluster =
                    fat_reader.get_next_cluster(current_cluster, Arc::clone(block_device));
                if current_cluster >= END_CLUSTER {
                    break;
                }
                current_sector = manager_reader.first_sector_of_cluster(current_cluster);
            } else {
                current_sector += 1; //没读完一个簇，直接进入下一扇区
            }
        }
        read_size
    }

    /// 以偏移量写文件，这里会对fat和manager加读锁
    pub fn write_at(
        &self,
        offset: usize,
        buf: &[u8],
        manager: &Arc<RwLock<FAT32Manager>>,
        fat: &Arc<RwLock<FAT>>,
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        // 获取共享锁
        let manager_reader = manager.read();
        let fat_reader = fat.read();
        let bytes_per_sector = manager_reader.bytes_per_sector() as usize;
        let bytes_per_cluster = manager_reader.bytes_per_cluster() as usize;
        let mut current_off = offset;
        let end: usize;
        if self.is_dir() {
            let size = bytes_per_cluster
                * fat_reader.count_cluster_num(self.first_cluster() as u32, block_device.clone())
                    as usize;
            end = offset + buf.len().min(size); // DEBUG:约束上界
        } else {
            // 从偏移量/缓冲区长度之和和设定的size中取最小值
            end = (offset + buf.len()).min(self.size as usize);
        }
        let (c_clu, c_sec, _) =
            self.get_pos(offset, manager, &manager_reader.get_fat(), block_device);
        // 找到当前的cluster和sector，我们这里应该是一样的
        let mut current_cluster = c_clu;
        let mut current_sector = c_sec;
        let mut write_size = 0usize;
        // println!("in write_at curr_sec:{}",current_sector);

        loop {
            // 将偏移量向上对齐扇区大小(一般是512)
            let mut end_current_block = (current_off / bytes_per_sector + 1) * bytes_per_sector;
            end_current_block = end_current_block.min(end);

            // 写
            let block_write_size = end_current_block - current_off;
            // println!("write cache: current_sector = {}", current_sector);
            if self.is_dir() {
                get_info_cache(
                    // 目录项通过infocache访问
                    current_sector,
                    Arc::clone(block_device),
                    CacheMode::READ,
                )
                .write()
                .modify(0, |data_block: &mut DataBlock| {
                    let src = &buf[write_size..write_size + block_write_size];
                    let dst = &mut data_block
                        [current_off % BLOCK_SZ..current_off % BLOCK_SZ + block_write_size];
                    dst.copy_from_slice(src);
                });
            } else {
                get_block_cache(current_sector, Arc::clone(block_device), CacheMode::READ)
                    .write()
                    .modify(0, |data_block: &mut DataBlock| {
                        let src = &buf[write_size..write_size + block_write_size];
                        let dst = &mut data_block
                            [current_off % BLOCK_SZ..current_off % BLOCK_SZ + block_write_size];
                        dst.copy_from_slice(src);
                    });
            }
            // 更新读取长度
            write_size += block_write_size;
            if end_current_block == end {
                break;
            }
            // 更新索引参数
            current_off = end_current_block;
            if current_off % bytes_per_cluster == 0 {
                // 读完一个簇
                // println!("finish writing a cluster");

                // 查询下一个簇
                current_cluster =
                    fat_reader.get_next_cluster(current_cluster, Arc::clone(block_device));
                if current_cluster >= END_CLUSTER {
                    panic!("END_CLUSTER");
                } //没有下一个簇
                  // 计算所在扇区
                  // println!("write at current_cluster = {}", current_cluster);

                // 获取下一个簇的第一个扇区
                current_sector = manager_reader.first_sector_of_cluster(current_cluster);
                // println!("write at current_sector = {}", current_sector);
                //let mut guess = String::new();
                //std::io::stdin().read_line(&mut guess).expect("Failed to read line");
            } else {
                current_sector += 1; //没读完一个簇，直接进入下一扇区
            }
        }
        write_size
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as usize as *const u8, DIRENT_SZ) }
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as usize as *mut u8, DIRENT_SZ) }
    }
}

#[repr(packed)]
#[allow(unused)]
#[derive(Clone, Copy, Debug)]
// use Unicode !!!
// 如果是该文件的最后一个长文件名目录项，则将该目录项的序号与 0x40 进行“或（OR）运算”的结果写入该位置。
// 长文件名要有\0
pub struct LongDirEntry {
    order: u8,       // 属性字节位 删除时为0xE5
    name1: [u8; 10], // 长文件名 unicode 5characters
    attribute: u8,   // 长文件名目录性标志 should be 0x0F
    type_: u8,       // 系统保留
    check_sum: u8,   // 校验值
    name2: [u8; 12], // 长文件名 unicode 6characters
    zero: [u8; 2],   // 文件起始簇号
    name3: [u8; 4],  // 长文件名 unicode 2characters
}

impl From<&[u8]> for LongDirEntry {
    fn from(bytes: &[u8]) -> Self {
        Self {
            order: bytes[0],
            name1: clone_into_array(&bytes[1..11]), // 5characters
            attribute: bytes[11],                   // should be 0x0F
            type_: bytes[12],                       //
            check_sum: bytes[13],
            name2: clone_into_array(&bytes[14..26]), // 6characters
            zero: clone_into_array(&bytes[26..28]),
            name3: clone_into_array(&bytes[28..32]), // 2characters
        }
    }
}

impl LongDirEntry {
    pub fn empty() -> Self {
        Self {
            order: 0,       // 删除时为0xE5
            name1: [0; 10], // 5characters
            attribute: 0,   // should be 0x0F
            type_: 0,       //
            check_sum: 0,
            name2: [0; 12], // 6characters
            zero: [0; 2],
            name3: [0; 4], // 2characters
        }
    }

    pub fn attribute(&self) -> u8 {
        self.attribute
    }

    pub fn is_empty(&self) -> bool {
        if self.order == 0x00 {
            true
        } else {
            false
        }
    }

    #[allow(unused)]
    pub fn is_valid(&self) -> bool {
        if self.order == 0xE5 {
            false
        } else {
            true
        }
    }

    pub fn is_deleted(&self) -> bool {
        if self.order == 0xE5 {
            true
        } else {
            false
        }
    }
    /* 上层要完成对namebuffer的填充，注意\0，以及checksum的计算 */
    /* 目前只支持英文，因此传入ascii */
    pub fn initialize(&mut self, name_buffer: &[u8], order: u8, check_sum: u8) {
        let ord = order;
        //println!("** initialize namebuffer = {:?}", name_buffer);
        //if is_last { ord = ord | 0x40 }
        let mut name1: [u8; 10] = [0; 10];
        let mut name2: [u8; 12] = [0; 12];
        let mut name3: [u8; 4] = [0; 4];
        let mut end_offset = 0;
        for i in 0..5 {
            if end_offset == 0 {
                name1[i << 1] = name_buffer[i];
                if name_buffer[i] == 0 {
                    end_offset = i;
                }
            } else {
                name1[i << 1] = 0xFF;
                name1[(i << 1) + 1] = 0xFF;
            }
        }
        for i in 5..11 {
            if end_offset == 0 {
                name2[(i - 5) << 1] = name_buffer[i];
                if name_buffer[i] == 0 {
                    end_offset = i;
                }
            } else {
                name2[(i - 5) << 1] = 0xFF;
                name2[((i - 5) << 1) + 1] = 0xFF;
            }
        }
        for i in 11..13 {
            if end_offset == 0 {
                name3[(i - 11) << 1] = name_buffer[i];
                if name_buffer[i] == 0 {
                    end_offset = i;
                }
            } else {
                name3[(i - 11) << 1] = 0xFF;
                name3[((i - 11) << 1) + 1] = 0xFF;
            }
        }
        *self = Self {
            order: ord,
            name1,
            attribute: ATTRIBUTE_LFN,
            type_: 0,
            check_sum,
            name2,
            zero: [0u8; 2],
            name3,
        }
    }

    pub fn clear(&mut self) {
        //self.order = 0xE5;
    }

    pub fn delete(&mut self) {
        self.order = 0xE5;
    }

    /* 获取长文件名，此处完成unicode至ascii的转换，暂不支持中文！*/
    // 这里直接将几个字段拼合，不考虑填充字符0xFF等
    // 需要和manager的long_name_split配合使用
    pub fn get_name_raw(&self) -> String {
        let mut name = String::new();
        let mut c: u8;
        for i in 0..5 {
            c = self.name1[i << 1];
            //if c == 0 { return name }
            name.push(c as char);
        }
        for i in 0..6 {
            c = self.name2[i << 1];
            //if c == 0 { return name }
            name.push(c as char);
        }
        for i in 0..2 {
            c = self.name3[i << 1];
            //if c == 0 { return name }
            name.push(c as char);
        }
        return name;
    }

    pub fn get_name_format(&self) -> String {
        // 拼接文件名 // 过滤 FF 和 00
        let name1: String = self
            .name1
            .iter()
            .filter(|x| **x != 0xFF && **x != 0)
            .map(|x| *x as char)
            .collect();
        let name2: String = self
            .name2
            .iter()
            .filter(|x| **x != 0xFF && **x != 0)
            .map(|x| *x as char)
            .collect();
        let name3: String = self
            .name3
            .iter()
            .filter(|x| **x != 0xFF && **x != 0)
            .map(|x| *x as char)
            .collect();
        format!("{}{}{}", name1, name2, name3)
    }

    #[allow(unused)]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as usize as *const u8, DIRENT_SZ) }
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as usize as *mut u8, DIRENT_SZ) }
    }
    pub fn get_order(&self) -> u8 {
        self.order
    }
    pub fn get_checksum(&self) -> u8 {
        self.check_sum
    }
}
