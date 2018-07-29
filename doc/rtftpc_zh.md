# rtftpc

## 描述

## 目标

## API
1. public rtftpc(Func<int, string, int> log) public rtftpc(Func<int, string, int> log, int timeout, int maxretry, int blksize)
    - 构造函数
    - Log: log输出接口;第一个参数为输出位置，-1表示新行输出，第二个参数为待输出字符串，返回值为输出位置；该接口最初用于ListBox的同行进度更新
    - timeout: 报文超时时间，默认 200 ms
    - maxretry: 报文重传次数，默认 10
    - blksize: 块大小，默认 1024 字节

2. public void get(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
    - 从服务器读文件
    - srvip: 服务器IP
    - srvport: 服务器端口，默认 69
    - remoteFile: 服务器文件名
    - localFile: 保存到的本地文件名
    - tftpmode: tftp模式

3. public void put(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
    - 上传文件到服务器
    - srvip: 服务器IP
    - srvport: 服务器端口，默认 69
    - remoteFile: 服务器文件名
    - localFile: 保存到的本地文件名
    - tftpmode: tftp模式

## 示例
    ```
    void tftpc_read_args()
    {
        saveconf();
        tftpc_ip = ((TextBox)texthash["tftpc_addr"]).Text;
        tftpc_tout = int.Parse(((TextBox)texthash["tftpc_timeout"]).Text);
        tftpc_retry = int.Parse(((TextBox)texthash["tftpc_retry"]).Text);
        tftpc_blksize = int.Parse(((TextBox)texthash["tftpc_blksize"]).Text);
        tftpc_lfile = text_tftpclfile.Text;
        tftpc_rfile = text_tftpcrfile.Text;
        tftpc = new rtftpc(tftpc_log_func, tftpc_tout, tftpc_retry, tftpc_blksize);
    }
    void tftpc_get_click(object sender, EventArgs e)
    {
        tftpc_read_args();
        Thread t = new Thread(() => tftpc.get(tftpc_ip, 69, tftpc_rfile, tftpc_rfile, Modes.octet));
        t.IsBackground = true;
        t.Start();
    }
    void tftpc_put_click(object sender, EventArgs e)
    {
        tftpc_read_args();
        Thread t = new Thread(() => tftpc.put(tftpc_ip, 69, Path.GetFileName(tftpc_lfile), tftpc_lfile, Modes.octet));
        t.IsBackground = true;
        t.Start();
    }
    ```