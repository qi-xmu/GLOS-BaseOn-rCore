#!/bin/bash

if test $# -ne 3 
then
echo Usage:
echo makesd [dev] [tmp] [apps] 
exit
fi  
SDCARD=$1
TMP_MOUNT=$2
USER_BIN=$3


sudo umount ${SDCARD}
sudo mkfs.vfat -F 32 ${SDCARD}
sudo mount ${SDCARD} ${TMP_MOUNT}
sudo cp -r ${USER_BIN}/* ${TMP_MOUNT}
sudo umount ${TMP_MOUNT}