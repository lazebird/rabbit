# rshell

## 描述
- 实现文件系统右键菜单接口

## 目标

## API
1. public rshell(string appname, string apppath, string menuname)
    - 构造函数
    - appname: 应用名称，用于注册表内显示
    - apppath: 可执行路径，用于右键执行程序
    - menuname: 右键菜单名称

2. public void reg_file()
    - 注册到所有文件

3. public void reg_dir()
    - 注册到所有目录

4. public void dereg_file()
    - 解除所有文件的注册

5. public void dereg_dir()
    - 解除所有目录的注册

6. public bool file_exist()
    - 是否已经注册到所有文件

7. public bool dir_exist()
    - 是否已经注册到所有目录

## 示例
    ```
    rshell sh;
    sh = new rshell("Rabbit", Application.ExecutablePath, "Add to Rabbit.http");
    CheckBox cb_http_shell = new CheckBox();
    if (sh.file_exist()) cb_http_shell.Checked = true;
    cb_http_shell.CheckedChanged += new EventHandler(http_shell_click);
    void http_shell_click(object sender, EventArgs e)
    {
        CheckBox cb = (CheckBox)sender;
        //httpd_log_func("I: current checked " + cb.Checked + " exist " + sh.file_exist());
        if (cb.Checked) { sh.reg_file(); sh.reg_dir(); }
        else { sh.dereg_file(); sh.dereg_dir(); }
    }
    ```