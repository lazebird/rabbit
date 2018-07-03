# ping

## Description

## Target

## API
1. public rping(Action<string> log)  
Constructor
Log: log output interface

2. public void start(string addr, int timeout, Action<int, PingReply> callback)  
Synchronous ping operation
Addr: ping address
Timeout: ping timeout in milliseconds
Callback: the callback function that handles the result. The first parameter is the request id, which is used to match the request and response. The second parameter is the ping result object.

3. public int start_async(string addr, int timeout, Action<int, PingReply> callback)  
Asynchronous ping operation
Addr: ping address
Timeout: ping timeout in milliseconds
Callback: the callback function that handles the result. The first parameter is the request id, which is used to match the request and response. The second parameter is the ping result object.

## Sample
```
rping ping = new rping(ping_log_func);
void ping_log_func(string msg)
{
    console.write(msg);
}
void ping_cb(int id, PingReply reply)
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
ping.start(addr, timeout, ping_cb);

void scan_reply(int id, PingReply reply)
{
    if (reply == null || reply.Status != IPStatus.Success)
    {
        ((Label)lbhash[id]).BackColor = Color.Red;
    }
}
int id = ping.start_async(addr, timeout, scan_reply);
```