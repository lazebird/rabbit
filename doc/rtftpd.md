# rtftpd

## Description

## Target

## API
1. rtftpd(Func<int, string, int> log)
    - Constructor
    - Log: log output interface; the first parameter is the output position, 0 means automatic position, the second parameter is the string to be output, and the return value is the output position; the interface is initially used for peer progress update of ListBox.

2. public void set_cwd(string path)
    - set current work directory
    - Path: folder path

3. public void start(int port, Hashtable opts)
    - Start the server
    - port: udp port, recommend 69
    - opts:
- timeout: Receive timeout, default 200 ms
         - maxretry: number of message retransmissions, default 10
         - blksize: block size, default 512 bytes; usually initiated by the client
         - qsize: file read and write queue size, large queue means high memory usage, and concurrency guarantee is also better, default 2000, usually no need to modify
         - qtout: file read and write queue timeout, large timeout means high fault tolerance, default 1000ms, usually no need to modify
         - override: The file overrides the write identifier, which is mainly valid when the client puts the request. When it is true, the file put by the client and the local conflict are allowed to be overwritten. Otherwise, the prompt file already exists.
         - fslog: The file system prints the log identifier. When it is true, it will additionally print the file read and write information for file system debugging.

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
    tftpd.start(69, ropt.parse_opts(text_tftpdopt.Text));
    ```