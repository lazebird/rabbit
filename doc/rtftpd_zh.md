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

3. public void start(int port, int timeout, int maxretry)
    - 启动服务器
    - port: UDP端口，建议 69
    - timeout: 收包超时，建议 200 ms
    - maxretry: 报文重传次数，建议 10

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
    tftpd.start(69, 200, 30);
    ```