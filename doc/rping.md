# ping

## Description

## Target

## API
1. public rping(Action<string> log, string addr) public rping(Action<string> log, string addr, int interval, int count, bool stoponloss)
    - Constructor
    - log: log output interface
    - addr: ping destination address
    - interval: ping interval, also used to be timeout,default 1000ms
    - count: ping send count, default -1, means infinit
    - stoponloss: stop ping if there is a pkt loss

2. public void start(Action<PingReply, object> callback, object data)
    - ping session start
    - callback: a callback function, which will be called for all ping replies; when ping stopped, callback called with PingReply=null
    - data: argument for callback

3. public void stop()
    - ping session stop

4. public override string ToString()
    - ping session tostring
    - return realtime session statistics as a string

## Sample
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