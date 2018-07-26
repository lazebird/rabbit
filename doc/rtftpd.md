# rtftpd

## Description

## Target

## API
1. public rtftpd(Func<int, string, int> log) public rtftpd(Func<int, string, int> log, int maxretry, int timeout)
    - Constructor
    - Log: log output interface; the first parameter is the output position, 0 means automatic position, the second parameter is the string to be output, and the return value is the output position; the interface is initially used for peer progress update of ListBox.

2. public void set_cwd(string path)
    - set current work directory
    - Path: folder path

3. public void start(int port, int timeout, int maxretry)
    - Start the server
    - port: udp port, recommend 69
    - timeout: receive timeout, recommend 200 ms
    - maxretry: packet retransimition times, recommend 10

4. public void stop()  
    - Stop the server while emptying the listening folder

## Sample
    ```
    rtftpd rtftpd = new rtftpd(rtftpd_log_func);
    rtftpd.set_cwd(t.Text);
    int rtftpd_log_func(int line, string msg)
    {
        return rtftpdlog.write(line, msg);
    }
    tftpd.start(69, 200, 30);
    ```