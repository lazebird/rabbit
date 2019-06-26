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

3. public void start(int port, Hashtable opts)
    - Start rhttpd
    - Port: http port number
    - opts:
        - autoindex: Open the file if there is an index.html or index.htm file under it when opening the directory.
        - videoplay: When the video file is opened, the player is enabled by default instead of downloading

4. public void stop()  
    - Stop rhttpd

5. public void add_dir(string path)  
    - Add a folder to the root directory
    - Path: folder path

6. public void del_dir(string path)  
    - Delete a folder from the root directory
    - Path: folder path

7. public void add_file(string path)  
    - Add files from the root directory
    - Path: file path

8. public void del_file(string path)  
    - Delete files from the root directory
    - Path: file path

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