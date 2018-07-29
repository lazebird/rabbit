# ping

## 描述

## 目标

## API
1. public rping(Action<string> log)  
    - 构造函数
    - log：log输出接口

2. public void start(string addr, int timeout, Action<PingReply, object> callback, object data)
    - 同步执行ping操作
    - addr：ping地址
    - timeout： ping超时时间，单位毫秒
    - callback：处理结果的回调函数，第一个参数为请求id，用于匹配请求和响应；第二个参数为ping结果对象
    - data: callback回调函数的传参

3. public void start_async(string addr, int timeout, Action<PingReply, object> callback, object data)
    - 异步执行ping操作
    - addr：ping地址
    - timeout： ping超时时间，单位毫秒
    - callback：处理结果的回调函数，第一个参数为请求id，用于匹配请求和响应；第二个参数为ping结果对象
    - data: callback回调函数的传参

## 示例
    ```
    rping ping = new rping(ping_log_func);
    void ping_log_func(string msg)
    {
        console.write(msg);
    }
    void ping_cb(PingReply reply, object data)
    {
        if (reply == null)  // exception
        {
        }
        else if (reply.Status == IPStatus.Success)
        {
            pinglog.write("来自 " + reply.Address + " 的回复: 字节=" + reply.Buffer.Length + " 毫秒=" + reply.RoundtripTime + " TTL=" + reply.Options.Ttl);
        }
        else
        {
            pinglog.write("请求超时。");
        }
    }
    ping.start(addr, timeout, ping_cb, null);

    void scan_reply(PingReply reply, object data)
    {
        if (reply != null && reply.Status == IPStatus.Success)
        {
            ((Label)data).BackColor = Color.Green;
        }
    }
    ping.start_async((new IPAddress(ipbytes)).ToString(), 1000, scan_reply, lb);
    ```