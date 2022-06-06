# GLOS Base On rCore

## 项目介绍

glos项目基于rCore-Tutorial-v3的基础上进行开发，目前只是完成了fat32文件系统的部分移植。
可以按照批处理方式运行应用程序。

## 项目计划

- 首先完善操作系统的各个功能。
  - 进程管理
  - 信号
  

## 项目说明

基本沿用了rCore的开发过程，其中添加了一些简化的方法，具体可以查看makefile。
一些常用的makefile变量存放在了makefile.in文件中。