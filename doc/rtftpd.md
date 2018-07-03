# tftpd

## Description

## Target

## API
1. public tftpd(Func<int, string, int> log)  
Constructor
Log: log output interface; the first parameter is the output position, 0 means automatic position, the second parameter is the string to be output, and the return value is the output position; the interface is initially used for peer progress update of ListBox.

2. public void add_dir(string dir)  
Add a folder to the root directory
Path: folder path

3. public void del_dir(string path)  
Delete a folder from the root directory
Path: folder path

4. public void start(int port)  
Start the server

5. public void stop()  
Stop the server while emptying the listening folder

## Sample
```
tftpd tftpd = new tftpd(tftpd_log_func);
tftpd.add_dir(t.Text);
int tftpd_log_func(int line, string msg)
{
    return tftpdlog.write(line, msg);
}
```