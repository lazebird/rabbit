
# <center>欢迎大家能加入项目开发，使其变得更加强大有趣!</center>

# Rabbit
Windows平台下的小工具集合。

## 动机
程序员经常会收集很多小工具，比如著名的HFS/tftpd/atkping等小工具，这些小工具很实用、很有趣、很经典。
但是这些小工具往往很零散，经常有一个app目录下面放满了这类小工具，这使得保存和使用这些工具变得麻烦。
另一方面很多小工具都太经典，以至于多年前就停止维护了，更加缺乏对新特性新需求的更新支持。
因此我建立了本项目，希望能够尽量多的将各种小功能加入到本项目中。
1. 一套代码解决项目开发的碎片化问题
2. 开源确保代码可以被任何人维护和更新，解决可持续性问题
3. 尽量统一UI，单一可执行程序，解决使用的碎片化问题

## 库API
库API用法参考项目下doc/{library name}.mk, 比如doc/ping.mk, 本文档主要对应用程序进行描述。

## 描述
1. ping工具，特点是支持Windows任务栏着色反馈ping状态。
2. IP扫描工具，使用ping接口进行扫描，检测网段中存在哪些IP。
3. HTTP服务器, 支持设置根目录和端口。
4. tftp服务器, 借由第三方tftp库实现的。
5. 支持中文和英语，但是中文资源文件没法整合到同一个执行文件中，因此当前发布的 **单执行文件版本仅支持英文**。
6. ...

## 发布
1. 使用ilmerge.exe可以将生成的执行文件和库一起打包成一个单独的可执行文件，命令参考如下
```
	..\..\..\tools\ILMerge.exe /targetplatform:v4 /ndebug /target:winexe /out:sRabbit.exe lazebird.rabbit.rabbit.exe lazebird.rabbit.common.dll lazebird.rabbit.ftp.dll lazebird.rabbit.ping.dll Microsoft.WindowsAPICodePack.dll Tftp.Net.dll lazebird.rabbit.dhcp.dll lazebird.rabbit.http.dll lazebird.rabbit.tftp.dll Microsoft.WindowsAPICodePack.Shell.dll zh-CN/lazebird.rabbit.rabbit.resources.dll
```

## 截图
1. ![Alt ?](/doc/Screenshots.PNG)  

## 待办
1. 添加APP自动升级功能？
2. 为程序员添加暗黑主题？
3. 更多小功能加入，比如ftpd/dhcpd...
