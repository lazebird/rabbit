# 文件操作

## 描述
- 实现了部分文件系统相关接口

## 目标

## API
1. public rfs(Action<string> log)
    - 构造函数
    - log: log打印接口

2. public int addfile(Hashtable hs, string vpath, string rpath)
    - 添加一个文件到虚拟文件系统，虚拟系统信息保存在哈希表中
    - hs: 虚拟系统哈希表
    - vpath: 虚拟系统文件路径
    - rpath: 文件真实路径

3. public int delfile(Hashtable hs, string vpath, string rpath)
    - 从虚拟文件系统中删除一个文件
    - hs: 虚拟系统哈希表
    - vpath: 虚拟系统文件路径
    - rpath: 文件真实路径

4. public int adddir(Hashtable hs, string vpath, string rpath, bool recursive)
    - 添加一个目录到虚拟文件系统，虚拟系统信息保存在哈希表中
    - hs: 虚拟系统哈希表
    - vpath: 虚拟系统文件路径
    - rpath: 文件真实路径
    - recursive: 是否递归遍历添加子目录

5. public int deldir(Hashtable hs, string vpath, string rpath)
    - 从虚拟文件系统中删除一个目录
    - hs: 虚拟系统哈希表
    - vpath: 虚拟系统文件路径
    - rpath: 文件真实路径

6. public void readstream(Stream fs, rqueue q, int maxblksz)
    - 读输入流并保存到队列
    - fs: 输入流
    - q: 资源队列，用于保存读取的数据
    - maxblksz: 读取/保存的块大小

7. public void writestream(Stream output, rqueue q, string path)
    - 将队列数据写入到输出流
    - output: 输出流
    - q: 资源队列，保存了数据
    - path: 文件路径/输出流信息，仅用作打印显示


## 示例
    ```
    void file_load(HttpListenerResponse response, string path)
    {
        Stream output = response.OutputStream;
        response.ContentType = get_mime(get_suffix(path));
        //log("Info: response suffix " + get_suffix(uri) + " ContentType " + response.ContentType);
        FileStream fs = new FileStream((string)((rfile)fhash[path]).path, FileMode.Open, FileAccess.Read);
        response.ContentLength64 = fs.Length;
        rqueue q = new rqueue(10); // 10 * 10M, max memory used 100M
        new Thread(() => rfs.readstream(fs, q, 10000000)).Start();    // 10000000, max block size 10M
        new Thread(() => rfs.writestream(output, q, fs.Name)).Start();
    }
    ```