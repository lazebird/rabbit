# rhttpd

## Description

## Target

## API
1. public rhttpd(Action<string> log)  
    - 构造函数
    - log：log输出接口

2. public void init_mime(string value)  
    - 初始化MIME映射关系
    - value：MIME映射字符串，例如：
    ```
    string mime = "";
    mime += ".svg:image/svg+xml;";
    mime += ".html:text/html;";
    mime += ".htm:text/html;";
    mime += ".js:application/x-javascript;";
    mime += ".css:text/css;";
    mime += ".mp4:video/mpeg4;";
    mime += ".mpeg:video/mpg;";
    mime += ".avi:video/avi;";
    mime += ".mp3:audio/mp3;";
    mime += ".mid:audio/mid;";
    mime += ".jpg:application/x-jpg;";
    mime += ".jpeg:image/jpeg;";
    mime += ".img:application/x-img;";
    mime += ".ico:image/x-icon;";
    mime += ".png:image/png;";
    mime += "*:application/octet-stream;";
    ```

3. public void start(int port, Hashtable opts)
    - 启动rhttpd
    - port：http端口号
    - opts:
        - autoindex: 打开目录时如果其下有index.html或index.htm文件，自动打开该文件
        - videoplay：打开视频文件时默认启用播放器打开，而不是下载

4. public void stop()  
    - 停止rhttpd

5. public void add_dir(string path)  
    - 添加文件夹到根目录中
    - path：文件夹路径

6. public void del_dir(string path)  
    - 从根目录中删除文件夹
    - path：文件夹路径

7. public void add_file(string path)  
    - 从根目录中添加文件
    - path：文件路径

8. public void del_file(string path)  
    - 从根目录中删除文件
    - path：文件路径

## Sample
    ```
    rhttpd rhttpd = new rhttpd(rhttpd_log_func);
    rhttpd.init_mime(myconf.get("mime"));
    rhttpd.set_root(".");
    Hashtable http_opts = ropt.parse_opts(text_httpopt.Text);
    rhttpd.start(80, http_opts);
    void rhttpd_log_func(string msg)
    {
        console.write(msg);
    }
    ```