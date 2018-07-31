# rhttpd

## Description

## Target

## API
1. public rhttpd(Action<string> log)  
    - Constructor
    - log：Log output interface

2. public void init_mime(string value)  
    - Initialize MIME mapping
    - value：MIME mapping string, for example:
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
    - Start rhttpd
    - Port: http port number

4. public void stop()  
    - Stop rhttpd

5. public void set_root(string path)  deleted
    - Set the server root directory, the default is empty directory
    - use add_file/add_dir instead for all files/subdirs in root

6. public void add_dir(string path)  
    - Add a folder to the root directory
    - Path: folder path

7. public void del_dir(string path)  
    - Delete a folder from the root directory
    - Path: folder path

8. public void add_file(string path)  
    - Add files from the root directory
    - Path: file path

9. public void del_file(string path)  
    - Delete files from the root directory
    - Path: file path

## Sample
    ```
    rhttpd rhttpd = new rhttpd(rhttpd_log_func);
    rhttpd.init_mime(myconf.get("mime"));
    rhttpd.set_root(".");
    rhttpd.start(8000);
    void rhttpd_log_func(string msg)
    {
        console.write(msg);
    }
    ```