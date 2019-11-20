# 目录
[TOC]

# 测试环境
## 客户端
- debian 1.31 /tmp/ tftp-hpa

## 服务器
- win10 1.106 /1.31/.../rootfs.ubi
- YaTFTPSvr v1.0a 
- sRabbit 2.0.0.0   https://github.com/lazebird/rabbit/releases

# 测试结果
## YaTFTPSvr
1. 
    - 实际会话端口随机
    ``` shell
    lqy-vm:/tmp#time tftp 192.168.1.106 -c get rootfs.ubi
    
    real    3m34.659s
    user    0m4.412s
    sys     0m12.720s
    lqy-vm:/tmp# ls -l
    total 108796
    -rw-rw-rw- 1 root root 111399061 Jul 23 14:39 rootfs.ubi
    lqy-vm:/tmp# 
    ```
    - 速度约519KBps
2. 
    - 实际会话端口随机
    ``` shell
    lqy-vm:/tmp# time tftp 192.168.1.106 -c get rootfs.ubi
    
    real    3m51.334s
    user    0m4.228s
    sys     0m11.976s
    lqy-vm:/tmp# ls -l
    total 168696
    -rw-rw-rw- 1 root root  61335766 Jul 24 11:44 firmware
    -rw-rw-rw- 1 root root 111399061 Jul 24 11:52 rootfs.ubi
    lqy-vm:/tmp# 
    ```
    - 速度约484KBps

3. 
    - 实际会话端口随机
    ``` shell
    lqy-vm:/tmp# time tftp 192.168.1.106 -c get rootfs.ubi
    real    3m51.306s
    user    0m4.224s
    sys     0m12.656s
    lqy-vm:/tmp# ls -l
    total 112296
    -rw-rw-rw- 1 root root 114984705 Jul 30 17:01 rootfs.ubi
    lqy-vm:/tmp#  
    ```
    - 速度约497KBps
## sRabbit
1. 
    - 实际会话端口为69
    - 10*300ms retry 114
    ``` shell
    lqy-vm:/tmp# time tftp 192.168.1.106 -c get rootfs.ubi
    
    real    1m39.676s
    user    0m4.292s
    sys     0m9.796s
    lqy-vm:/tmp# 
    ```
    - 速度约1.117MBps

2. 
    - 实际会话端口为69
    ``` shell
    lqy-vm:/tmp# rm -f *
    lqy-vm:/tmp# time tftp 192.168.1.106 -c get rootfs.ubi
    
    real    1m21.198s
    user    0m4.004s
    sys     0m10.576s
    lqy-vm:/tmp# ls -l
    total 108792
    -rw-rw-rw- 1 root root 111399061 Jul 23 14:58 rootfs.ubi
    lqy-vm:/tmp# 
    ```
    - 速度约为1.37MBps

3. 
    - 实际会话端口随机
    - 30*200ms  retry 108
    ``` shell
    lqy-vm:/tmp# time tftp 192.168.1.106 -c get rootfs.ubi
    real    1m25.402s
    user    0m3.756s
    sys     0m10.008s
    lqy-vm:/tmp# ls -l
    total 168692
    -rw-rw-rw- 1 root root  61335766 Jul 24 11:44 firmware
    -rw-rw-rw- 1 root root 111399061 Jul 24 11:46 rootfs.ubi
    lqy-vm:/tmp# 
    ```
    - 速度约为1.31MBps
4. 
    - 实际会话端口随机
    - 30*200ms  retry 106
    ``` shell
     lqy-vm:/tmp# time tftp 192.168.1.106 -c get rootfs.ubi
    real    1m23.478s
    user    0m3.788s
    sys     0m11.180s
    lqy-vm:/tmp# ls -l
    total 112292
    -rw-rw-rw- 1 root root 114984705 Jul 30 17:04 rootfs.ubi
    lqy-vm:/tmp# 
    ```
    - 速度约为1.38MBps

