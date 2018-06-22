# ping
ping with state shown in Windows taskbar.

DESCRIPTION:
1. a simple ping app on windows.
2. it can show last 5 reply state on taskbar.

DISTRIBUTION:
1. copy ping.exe and Microsoft.WindowsAPICodePack.dll(optional) Microsoft.WindowsAPICodePack.Shell.dll(optional) out and run ping.exe directly.
2. use ilmerge.exe combine ping.exe and dlls like below, copy and run sping.exe directly.  
	.\ILMerge.exe /targetplatform:v4 /ndebug /target:winexe  /out:sping.exe ping.exe  Microsoft.WindowsAPICodePack.dll Microsoft.WindowsAPICodePack.Shell.dll

SCREENSHOTS:
1. ![Alt ?](./Screenshots.PNG)  

TODO:
1. add version management and auto update function?
2. more other small functions/tools, like httpd/tftpd/ftpd...

WELCOME ALL FRIENDS TO JOIN THIS PROJECT AND MAKE IT MORE INTERESTING.

