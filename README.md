# Rabbit
A collection of windows small tools.

DESCRIPTION:
1. a ping app, with state shown in taskbar.
2. a scan app, use ping to check if a ip used.
3. a http server, support set root dir and port.
4. a tftp server, implemented with a third party library.
5. support chinese simplified and english, but i don't known how to combine them into a single exe file. current combined exe file can only support english.
6. ...

DISTRIBUTION:
1. use ilmerge.exe combine exe and dlls like below, copy and run sRabbit.exe directly.  
	.\ILMerge.exe /targetplatform:v4 /ndebug /target:winexe  /out:sRabbit.exe lazebird.rabbit.rabbit.exe  Microsoft.WindowsAPICodePack.dll Microsoft.WindowsAPICodePack.Shell.dll ... dlls

SCREENSHOTS:
1. ![Alt ?](./Screenshots.PNG)  

TODO:
1. add version management and auto update function?
2. add a light dark theme for programmers.
3. more other small functions/tools, like ftpd/dhcpd...

WELCOME ALL FRIENDS TO JOIN THIS PROJECT AND MAKE IT MORE INTERESTING.

