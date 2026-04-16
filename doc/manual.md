# Rabbit
- A collection of gadgets under the Windows platform.
- Function list: ping, IP scan, HTTP server, TFTP server, TFTP client, timed reminder, LAN chat

## table of Contents
[TOC]

## General
### Configuration Item
- The default Esc key will exit the current window
- The default Enter key will execute the Start/Stop button in the current tab.
- The interaction in the program is mainly a mouse click/double click except for the explicit button

## Settings (Setting)
### Configuration Item
- Language: language selection, currently invalid
- System Tray: Activate the system tray icon so that the program can be minimized to the system tray; this function conflicts with the taskbar progress bar, the effect can not be both
- Home: Click to access the project home page
- Profile: Click to access the program configuration file directory
- Help: Click to access this manual
- Version: Program version information, click to update automatically
- restartprompt: Whether to restart the upgrade when the version is upgraded, the default is not prompted; this field must manually modify the configuration file, only for problem location

## Ping
### Configuration Item
- Addr.: Ping's address, which can be an IP or a domain name
- interval: Ping period, in milliseconds, default 1000ms
- count: number of pings, default -1, indicating continuous ping
- taskbar: taskbar status, when true, indicates that the taskbar is available; default is true
- log: log file name, which can be a relative path file name, or an absolute path file name. The default is empty, not writing files.

## IP Scan (Scan)
### Configuration Item
- IP: The range of scanned IP addresses. The previous box is the complete starting IP, and the last box is the last byte value of the ending IP.
- filter: display filtering options, true means only display reachable IP, false means show all IP; default is true

## HTTP Server (HTTPD)
### Configuration Item
- Port: HTTP server port, default is 8000
- shell: Whether to enable the file system right-click shell, it is not enabled by default; uncheck the delete right-click menu item
- index: Whether to enable automatic indexing of directories, that is, when accessing a directory, if there is an index file in the directory, the directory structure information is not loaded, but the index file is directly loaded instead.

## TFTP Server (TFTPD)
### Configuration Item
- +: Add the TFTP server directory. After clicking it, the directory selection box will pop up. After selecting it, the new directory will be used as the current server working directory by default.
- -: Delete current working directory
- timeout: Server packet transmission timeout configuration, in milliseconds, default 200ms
- retry: The number of retransmissions of the server packet. The default retransmission is 30 times.
- override: Whether to automatically overwrite when the file is uploaded, the default is no, false
- qsize: producer consumer queue size, large queues are conducive to concurrency when there is enough memory; files in the network share path can also optimize performance through large queues
- Click on the directory entry event: Select directory as current server working directory
- Double-click the directory entry event: Pop-up directory selection box, modify the directory entry to the new path

## TFTP Client (TFTPC)
### Configuration Item
- IP: Server IP Address
- timeout: Server packet transmission timeout configuration, in milliseconds, default 200ms
- retry: The number of retransmissions of the server packet. The default retransmission is 30 times.
- blksize: the block size of the transfer, in bytes, defaults to 1468 bytes
- Put input box: Double-click to select the local file as the put file
- Get input box: Enter the remote file name to be obtained

## Planning reminder (PLAN)
### Configuration Item
- Date: Planned start date
- Time: Planned start time
- Repeat: Plan repeated values ​​and units
- Text box: Plan name to ensure uniqueness
- override: If the newly added plan conflicts with the previous one, it is automatically overwritten and the default is not overwritten.
- -: Delete the plan; delete the plan based on the plan name
- +: Add a plan; add a plan based on the plan name
- Double-click the plan item: Export the plan item information to the input box; if you need to delete a plan, you can directly enter the plan name in the text box and click delete, or double-click on the corresponding plan, and then click delete; modify the plan needs to delete first Re-add
- A black screen reminder box will pop up when the plan expires. The plan name is displayed above. The default is 3 minutes. You can turn off the reminder by double-clicking the plan name on the screen.

## LAN chat (CHAT)
### Configuration Item
- Name text box: as a nickname for chat; default use account name @machine name as chat name
- Start/Stop: Chat service start/stop button, the default will automatically start with the APP startup
- Broadcast address: The destination address for broadcast discovery and notification. The default is 255.255.255.255. When there are multiple NICs/network segments, it is recommended to manually configure the subnet broadcast address, such as the common 192.168.1.255, in order to avoid network segment selection errors.
- Notify: Click to send a broadcast notification
- Refresh: Click to refresh the current LAN online user information; online user information will contain your own information
- User information click: Start a chat session with the user