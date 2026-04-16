# rtftpd

## 描述

## 目标

## API
1. public rtftpd(Func<int, string, int> log)
    - 构造函数
    - log：log输出接口;第一个参数为输出位置，-1表示新行输出，第二个参数为待输出字符串，返回值为输出位置；该接口最初用于ListBox的同行进度更新。

2. public void set_cwd(string path)
    - 设置工作路径
    - path：文件夹路径

3. public void start(int port, Hashtable opts)
    - 启动服务器
    - port: UDP端口，建议 69
    - opts:
        - timeout: 收包超时，默认 200 ms
        - maxretry: 报文重传次数，默认 10
        - blksize: 块大小，默认512字节；通常由客户端发起协商
        - qsize：文件读写队列大小，队列大意味着内存占用高，同时并发性保证也更好，默认2000，通常无需修改
        - qtout：文件读写队列超时，超时大意味着容错性高，默认1000ms，通常无需修改
        - override：文件覆盖写标识，主要在客户端put请求时生效，为true时客户端put的文件和本地冲突时允许覆盖，否则提示文件已经存在
        - fslog：文件系统打印log的标识，为true时会额外打印文件读写信息，用于文件系统调试

4. public void stop()  
    - 停止服务器

## 示例
    ```
    rtftpd rtftpd = new rtftpd(rtftpd_log_func);
    rtftpd.set_cwd(t.Text);
    int rtftpd_log_func(int line, string msg)
    {
        return rtftpdlog.write(line, msg);
    }
    tftpd.start(69, ropt.parse_opts(text_tftpdopt.Text));
    ```