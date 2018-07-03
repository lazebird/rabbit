# httpd

## Description

## Target

## API
1. public httpd(Action<string> log)  
构造函数
log：log输出接口

2. public void init_mime(string value)  
初始化MIME映射关系
value：MIME映射字符串，例如：
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

3. public bool start(int port)  
启动httpd
port：http端口号

4. public void stop()  
停止httpd

5. public void set_root(string path)  
设置服务器根目录，默认为空目录

6. public void add_dir(string path)  
添加文件夹到根目录中
path：文件夹路径

7. public void del_dir(string path)  
从根目录中删除文件夹
path：文件夹路径

8. public void add_file(string path)  
从根目录中添加文件
path：文件路径

9. public void del_file(string path)  
从根目录中删除文件
path：文件路径

## Sample
```
httpd httpd = new httpd(httpd_log_func);
httpd.init_mime(myconf.get("mime"));
httpd.set_root(".");
httpd.start(8000);
void httpd_log_func(string msg)
{
    console.write(msg);
}
```