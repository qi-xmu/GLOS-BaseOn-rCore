use alloc::{sync::Arc, vec::Vec};

use crate::{
    block_cache::{get_info_cache, CacheMode},
    println, BlockDevice, BAD_CLUSTER, END_CLUSTER, FATENTRY_PER_SEC, FREE_CLUSTER,
};

// 常驻内存，不作一一映射
#[allow(unused)]
#[derive(Clone, Copy)]
pub struct FAT {
    fat1_sector: u32, // FAT1的起始扇区
    fat2_sector: u32, // FAT2的起始扇区
    n_sectors: u32,   // 大小
    n_entry: u32,     // 表项数量
}

impl FAT {
    pub fn new(fat1_sector: u32, fat2_sector: u32, n_sectors: u32, n_entry: u32) -> Self {
        Self {
            fat1_sector,
            fat2_sector,
            n_sectors,
            n_entry,
        }
    }

    /* 计算簇对应表项的位置：sector和offset */
    fn calculate_pos(&self, cluster: u32) -> (u32, u32, u32) {
        // 返回sector号和offset
        // 前为FAT1的扇区号，后为FAT2的扇区号，最后为offset
        let fat1_sec = self.fat1_sector + cluster / FATENTRY_PER_SEC;
        let fat2_sec = self.fat2_sector + cluster / FATENTRY_PER_SEC;
        let offset = 4 * (cluster % FATENTRY_PER_SEC);
        (fat1_sec, fat2_sec, offset)
    }

    /* 搜索下一个可用簇 */
    // caller需要确定有足够的空闲簇，这里不作越界检查
    pub fn next_free_cluster(
        &self,
        current_cluster: u32,
        block_device: Arc<dyn BlockDevice>,
    ) -> u32 {
        let mut curr_cluster = current_cluster + 1;
        loop {
            #[allow(unused)]
            let (fat1_sec, fat2_sec, offset) = self.calculate_pos(curr_cluster);
            // 查看当前cluster的表项
            let entry_val =
                get_info_cache(fat1_sec as usize, block_device.clone(), CacheMode::READ)
                    .read()
                    .read(offset as usize, |&entry_val: &u32| entry_val);
            if entry_val == FREE_CLUSTER {
                break;
            } else {
                curr_cluster += 1;
            }
        }
        curr_cluster & 0x0FFFFFFF
    }

    /// 查询当前簇的下一个簇
    pub fn get_next_cluster(&self, cluster: u32, block_device: Arc<dyn BlockDevice>) -> u32 {
        // 需要对损坏簇作出判断
        // 及时使用备用表
        // 无效或未使用返回0
        let (fat1_sec, fat2_sec, offset) = self.calculate_pos(cluster);
        //println!("fat1_sec={} offset = {}", fat1_sec, offset);
        let fat1_rs = get_info_cache(fat1_sec as usize, block_device.clone(), CacheMode::READ)
            .read()
            .read(offset as usize, |&next_cluster: &u32| next_cluster);
        let fat2_rs = get_info_cache(fat2_sec as usize, block_device.clone(), CacheMode::READ)
            .read()
            .read(offset as usize, |&next_cluster: &u32| next_cluster);
        if fat1_rs == BAD_CLUSTER {
            if fat2_rs == BAD_CLUSTER {
                0
            } else {
                fat2_rs & 0x0FFFFFFF
            }
        } else {
            fat1_rs & 0x0FFFFFFF
        }
    }

    pub fn set_end(&self, cluster: u32, block_device: Arc<dyn BlockDevice>) {
        self.set_next_cluster(cluster, END_CLUSTER, block_device);
    }

    /* 设置当前簇的下一个簇 */
    pub fn set_next_cluster(
        &self,
        cluster: u32,
        next_cluster: u32,
        block_device: Arc<dyn BlockDevice>,
    ) {
        // 同步修改两个FAT
        // 注意设置末尾项为 0x0FFFFFF8
        //assert_ne!(next_cluster, 0);
        let (fat1_sec, fat2_sec, offset) = self.calculate_pos(cluster);
        get_info_cache(fat1_sec as usize, block_device.clone(), CacheMode::WRITE)
            .write()
            .modify(offset as usize, |old_clu: &mut u32| {
                *old_clu = next_cluster;
            });
        get_info_cache(fat2_sec as usize, block_device.clone(), CacheMode::WRITE)
            .write()
            .modify(offset as usize, |old_clu: &mut u32| {
                *old_clu = next_cluster;
            });
    }

    /* 获取某个文件的指定cluster */
    pub fn get_cluster_at(
        &self,
        start_cluster: u32,
        index: u32,
        block_device: Arc<dyn BlockDevice>,
    ) -> u32 {
        // 如果有异常，返回0
        //println!("** get_cluster_at index = {}",index);
        let mut cluster = start_cluster;
        #[allow(unused)]
        for i in 0..index {
            //print!("in fat curr cluster = {}", cluster);
            cluster = self.get_next_cluster(cluster, block_device.clone());
            //println!(", next cluster = {:X}", cluster);
            if cluster == 0 {
                break;
            }
        }
        cluster & 0x0FFFFFFF
    }

    pub fn final_cluster(&self, start_cluster: u32, block_device: Arc<dyn BlockDevice>) -> u32 {
        let mut curr_cluster = start_cluster;
        assert_ne!(start_cluster, 0);
        loop {
            let next_cluster = self.get_next_cluster(curr_cluster, block_device.clone());
            //println!("in fianl cl {};{}", curr_cluster, next_cluster);
            //assert_ne!(next_cluster, 0);
            if next_cluster >= END_CLUSTER || next_cluster == 0 {
                return curr_cluster & 0x0FFFFFFF;
            } else {
                curr_cluster = next_cluster;
            }
        }
    }

    pub fn get_all_cluster_of(
        &self,
        start_cluster: u32,
        block_device: Arc<dyn BlockDevice>,
    ) -> Vec<u32> {
        let mut curr_cluster = start_cluster;
        let mut v_cluster: Vec<u32> = Vec::new();
        loop {
            v_cluster.push(curr_cluster & 0x0FFFFFFF);
            let next_cluster = self.get_next_cluster(curr_cluster, block_device.clone());
            //println!("in all, curr = {}, next = {}", curr_cluster, next_cluster);
            //assert_ne!(next_cluster, 0);
            if next_cluster >= END_CLUSTER || next_cluster == 0 {
                return v_cluster;
            } else {
                curr_cluster = next_cluster;
            }
        }
    }
    // 计算文件的簇数量
    pub fn count_cluster_num(&self, start_cluster: u32, block_device: Arc<dyn BlockDevice>) -> u32 {
        if start_cluster == 0 {
            return 0;
        }
        let mut curr_cluster = start_cluster;
        let mut count: u32 = 0;
        loop {
            count += 1;
            let next_cluster = self.get_next_cluster(curr_cluster, block_device.clone());
            println!("next_cluster = {:X}", next_cluster);
            if next_cluster >= END_CLUSTER || next_cluster > 0xF000000 {
                return count;
            } else {
                curr_cluster = next_cluster;
            }
        }
    }
}
