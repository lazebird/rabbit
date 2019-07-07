# rconf

## Description
- Configuration item management library, support for state read, write, and storage management of TextBox, Button, CheckBox, TabControl, rstr, rint classes

## Target
- Configuration item management library, which is mainly used to decouple the winform UI configuration parameters and configuration file management mechanism and service modules, reducing configuration item complexity.

## API
1. public rconf(Func <string,string> initcb,Action <Hashtable> savecb)
    - Constructor
    - initcb: Initialize the callback function, pass in the parameter name, and pass the initial value of the parameter; the initial value can come from the default configuration, which can come from the saved file configuration, etc.
    - savecb: configuration file storage callback function, incoming configuration item hash table, configuration item hash table format: string parameter name - string parameter value

2. public bool bind(TextBox b, string name, Action <object,string> initcb, bool autosave_flag)
    - Parameter UI binding interface; after the parameter is bound by name, it will automatically find the initial value of the parameter from the system initcb and fill it into the UI, and call back the initcb function for further initialization.
    - b: UI object; this interface also derives interfaces for many other object types, including TextBox, Button, CheckBox, TabControl, rstr, rint, which are not listed here.
    - name: The name of the parameter, which uniquely identifies a parameter. It is also the unique identifier of the service to the parameter, and the interpretation of the business and UI.
    - initcb: Here is the callback when the parameter value is initialized, no additional processing can fill in the blank; usually only the button class parameters need to callback to further perform business startup and other activities
    - autosave_flag: Automatically saves the identifier. This flag takes effect on the set operation of the parameter. If it is true, after the parameter is set through the set interface, the save interface is automatically called to execute the action of writing the configuration file to the file.

3. public void save()
    - Profile storage interface
    - The external interface needs to call the interface when saving the configuration to the file. The implementation in the interface collects all the configuration items that have been bound, stores all the configuration information as a hash table, and calls the savecb callback to perform file storage processing.

4. public int getint(string name)/ public bool getbool(string name)/ public string getstr(string name)
    - Obtain parameter values ​​based on parameter names; the above multiple interface differences are implemented to convert parameter values ​​to different types, so that business modules can directly use valid values ​​without further conversion.
    - name: parameter name

5. public void set (string name, string val)
    - The parameter value sets the interface; the interface implementation will feedback the value to the corresponding UI element as needed; at the same time, check whether the autosave_flag of the parameter is valid. If it is valid, the configuration will be automatically written to the file.
    - name: parameter name
    - val: parameter value

## Sample
```
    rconf cfg;
    cfg = new rconf(conf_init, conf_save);
    cfg.bind(tabs, "tabindex", null, false);
    cfg.bind(text_pingaddr, "ping_addr", null, false);
    cfg.bind(text_pingopt, "ping_opt", null, false);
    cfg.bind(text_http_port, "http_port", null, false);
    cfg.bind(text_httpopt, "http_opt", null, false);
    cfg.bind(text_tftpdopt, "tftpd_opt", null, false);
    cfg.bind(text_scanstart, "scan_ipstart", null, false);
    cfg.bind(text_scanend, "scan_ipend", null, false);
    cfg.bind(text_scanopt, "scan_opt", null, false);
    cfg.bind(text_tftpcaddr, "tftpc_addr", null, false);
    cfg.bind(text_tftpcopt, "tftpc_opt", null, false);
    cfg.bind(text_planopt, "plan_opt", null, false);
    cfg.bind(btn_ping, "ping_btn", btn_init_cb, true);
    cfg.bind(btn_httpd, "http_btn", btn_init_cb, true);
    cfg.bind(tftpd_btn, "tftpd_btn", btn_init_cb, true);
    cfg.bind(text_chatname, "chatname", null, false);
    cfg.bind(text_chatntf, "chatbrdip", null, false);
    cfg.bind(new rstr("http_dirs", httpd_conf_get, httpd_conf_set), "http_dirs", null, false);
    cfg.bind(new rstr("plans", plan_conf_get, plan_conf_set), "plans", null, false);
    cfg.bind(cb_systray, "systray", null, false);
    cfg.bind(cb_autoupdate, "autoupdate", null, false);
    cfg.bind(new rstr("restartprompt", null, null), "restartprompt", null, false);
    cfg.bind(new rstr("tftpd_dirs", tftpd_conf_get, tftpd_conf_set), "tftpd_dirs", null, false);
    cfg.bind(new rstr("tftpd_dir_index", tftpd_conf_get, tftpd_conf_set), "tftpd_dir_index", null, false);      // depend on & must behind tftpd_dirs
    cfg.bind(new rstr("lang", null, null), "lang", null, false);
    cfg.bind(new rstr("mime", null, null), "mime", null, false);
```