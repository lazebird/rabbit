# ping

## 描述

## 目标

## API
1. public rping(Action<string> log)
    - 构造函数
    - log: 日志输出接口

2. public void start(string addr, Hashtable opts, Action<PingReply, object> callback, object data)
    - 启动ping会话
    - addr: ping目的地址
    - opts:
        - interval: ping间隔，也用作超时，默认1000ms
        - count: ping次数，默认-1，表示持续ping
        - stoponloss: 丢包时停止ping操作标识
    - callback: 回调函数，在每个ping响应处理时回调；同时在ping结束时也会回调，并且reply信息参数为null
    - data: 回调参数

3. public void stop()
    - 结束ping会话

4. public override string ToString()
    - tostring函数重写
    - 返回一个字符串，内容为实时的会话统计信息

## 示例
    ```
    void ping_cb(PingReply reply, object data)
    {
        if (reply == null)
        {
            ((Form)formhash["form"]).Text = "Rabbit";
            ((Button)btnhash["ping_btn"]).Text = Language.trans("开始");
        }
        else display_taskbar(reply.Status == IPStatus.Success);
        text_pingstat.Text = ping.ToString();
    }
    rping ping = new rping(ping_log_func);
    Hashtable ping_opts = ropt.parse_opts(text_pingopt.Text);
    ping.start(ping_addr, ping_opts, ping_cb, null);
    ```