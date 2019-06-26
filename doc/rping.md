# ping

## Description

## Target

## API
1. public rping(Action<string> log)
    - Constructor
    - log: log output interface

2. public void start(string addr, Hashtable opts, Action<PingReply, object> callback, object data)
    - ping session start
    - addr: ping destination address
    - opts:
        - interval: ping interval, also used to be timeout,default 1000ms
        - count: ping send count, default -1, means infinit
        - stoponloss: stop ping if there is a pkt loss
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
        text_pingstat.Text = ping.ToString();
    }
    rping ping = new rping(ping_log_func);
    Hashtable ping_opts = ropt.parse_opts(text_pingopt.Text);
    ping.start(ping_addr, ping_opts, ping_cb, null);
    ```