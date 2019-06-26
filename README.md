
# <center>All friends are welcome to join this project and make it more interesting!</center>

# Rabbit
A collection of windows small tools.

## INTENTION  
There are many small apps for programmers, like tftpd/hfs/atkping etc., they are very interesting and useful.
But it is scattered, and some of it is out of date, so I am here to integrate them together in this project.
I want to build a home for these apps here, make it long live here.
1. A set of code to solve the fragmentation problem of project development
2. Open source ensures that code can be maintained and updated by anyone to solve sustainability issues
3. Try to unify the UI, single executable program, solve the problem of fragmentation

## API
library APIs please refer to doc/{library name}.mk, such as doc/ping.mk, here is for the program.

## DESCRIPTION
1. a ping app, with state shown in taskbar.
2. a scan app, use ping to check if a ip used.
3. a http server, support set root dir and port.
4. a tftp server, implemented with a third party library.
5. support chinese simplified and english, but i don't known how to combine them into a single exe file. current combined exe file can **only support english**.
6. ...

## DISTRIBUTION
1. use ilmerge.exe combine exe and dlls like below, copy and run sRabbit.exe directly.  
	```
	..\..\..\tools\ILMerge.exe /targetplatform:v4 /ndebug /target:winexe /out:sRabbit.exe lazebird.rabbit.rabbit.exe /wildcards *.dll zh-CN/lazebird.rabbit.rabbit.resources.dll
	```

## SCREENSHOTS
1. ping ![Alt ?](/doc/resources/ping.PNG)  
2. scan ![Alt ?](/doc/resources/scan.PNG)  
3. httpd ![Alt ?](/doc/resources/httpd.PNG)  
4. tftpd ![Alt ?](/doc/resources/tftpd.PNG)  
5. tftpc ![Alt ?](/doc/resources/tftpc.PNG)  
6. plan ![Alt ?](/doc/resources/plan.PNG)  
7. setting ![Alt ?](/doc/resources/setting.PNG)  

## RELEASE NOTES
refer to /doc/releasenotes.md

## TODO
refer to /doc/todo.md
