# tftpd

## Description

## Target

## API
1. public tftpd(Func<int, string, int> log)
构造函数
log：log输出接口;第一个参数为输出位置，0表示自动占位置，第二个参数为待输出字符串，返回值为输出位置；该接口最初用于ListBox的同行进度更新。

2. public void add_dir(string dir)
添加文件夹到根目录中
path：文件夹路径

3. public void del_dir(string path)
从根目录中删除文件夹
path：文件夹路径

4. public void start(int port)
启动服务器

5. public void stop()
停止服务器，同时清空侦听的文件夹

## Sample
```
tftpd tftpd = new tftpd(tftpd_log_func);
tftpd.add_dir(t.Text);
int tftpd_log_func(int line, string msg)
{
    return tftpdlog.write(line, msg);
}
```