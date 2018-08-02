# rshell

## Description
- implement shell context api

## Target

## API
1. public rshell(string appname, string apppath, string menuname)
    - Constructor
    - appname: application name
    - apppath: executable path
    - menuname: shell context menu name

2. public void reg_file()
    - register to all files

3. public void reg_dir()
    - register to all directories

4. public void dereg_file()
    - deregister to all files

5. public void dereg_dir()
    - deregister to all directories

6. public bool file_exist()
    - if it has registered to all files

7. public bool dir_exist()
    - if it has registered to all directories

## Sample
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