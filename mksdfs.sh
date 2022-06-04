#!/bin/bash
sdcard=/dev/sda1
dir=/mnt
code=riscv-syscalls-testing/user/riscv64

sudo umount ${sdcard}
sudo mkfs.vfat -F 32 ${sdcard}
sudo mount $sdcard ${dir}
sudo cp -r ${code}/* ${dir}
sudo umount ${dir}