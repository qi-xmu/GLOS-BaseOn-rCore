use super::inode::ROOT_INODE;
use fat32::ATTRIBUTE_DIRECTORY;

pub fn read_test() {
    println!("test");
    let name = ROOT_INODE.get_name();
    println!("{}", name);
    for app in ROOT_INODE.ls_lite().unwrap() {
        if app.1 & ATTRIBUTE_DIRECTORY == 0 {
            print!("{} ", app.0);
        }
    }
}
