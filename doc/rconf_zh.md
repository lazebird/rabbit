# rconf

## 描述
- 配置项管理库，支持TextBox、Button、CheckBox、TabControl、rstr、rint类的状态读写和存储管理

## 目标
- 配置项管理库，主要用于将winform UI配置参数和配置文件管理机制以及业务模块解耦，降低配置项复杂度

## API
1. public rconf(Func<string, string> initcb, Action<Hashtable> savecb)
    - 构造函数
    - initcb：初始化回调函数，传入参数名称，传出参数初始值；该初始值可以来自默认配置，可以来自已经保存的文件配置等
    - savecb：配置文件存储回调函数，传入配置项哈希表，配置项哈希表格式为：string参数名-string参数值

2. public bool bind(TextBox b, string name, Action<object, string> initcb, bool autosave_flag)
    - 参数UI绑定接口；参数按照名称绑定后，会自动从系统initcb中找出参数初始值填充到UI，并按需回调initcb函数进行进一步初始化
    - b：UI对象；此接口也派生了诸多其他对象类型的接口，包括TextBox、Button、CheckBox、TabControl、rstr、rint类，此处不一一列出
    - name：参数名称，唯一标识一个参数，也是业务对参数的唯一识别码，实现业务和UI等的解耦
    - initcb：此处是在该参数值初始化时进行回调，无需额外处理可以填null；通常只有按钮类参数需要回调来进一步执行业务启动等行为
    - autosave_flag：自动保存标识，该标识对该参数的set操作生效；假设为true，在通过set接口设置该参数后，会自动调用save接口执行配置文件写入到文件的动作

3. public void save()
    - 配置文件存储接口
    - 外部需要将配置保存到文件时可以调用该接口，接口内实现会收集当前已经绑定的所有配置项，将所有配置信息存储为哈希表，调用savecb回调进行文件存储处理

4. public int getint(string name) / public bool getbool(string name) / public string getstr(string name)
    - 基于参数名获取参数值；上述多个接口差异是实现了参数值到不同类型的转换，便于业务模块直接使用有效值而无需再进一步转换
    - name：参数名

5. public void set(string name, string val)
    - 参数值设置接口；接口实现会将该值按需反馈到对应UI元素；同时检查参数的autosave_flag是否有效，有效的情况下会自动将配置写入到文件中
    - name：参数名
    - val：参数值

## 示例
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