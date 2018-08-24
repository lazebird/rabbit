# ping

## 描述

## 目标

## API
1. public rping(Action<string> log, string addr) public rping(Action<string> log, string addr, int interval, int count, bool stoponloss)
    - 构造函数
    - log: 日志输出接口
    - addr: ping目的地址
    - interval: ping间隔，也用作超时，默认1000ms
    - count: ping次数，默认-1，表示持续ping
    - stoponloss: 丢包时停止ping操作标识

2. public void start(Action<PingReply, object> callback, object data)
    - 启动ping会话
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
        if (ping != null) text_pingstat.Text = ping.ToString();
    }
    rping ping = new rping(ping_log_func, ping_addr, ping_interval, ping_count, ping_stoponloss);
    ping.start(ping_cb, null);
    ```