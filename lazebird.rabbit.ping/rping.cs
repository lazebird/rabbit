using System;
using System.Net.NetworkInformation;
using System.Threading;

namespace lazebird.rabbit.ping
{
    public class rping
    {
        Action<string> log;

        public rping(Action<string> log)
        {
            this.log = log;
        }
        public void start(string addr, int timeout, Action<PingReply, object> callback, object data)
        {
            ping_process(addr, timeout, callback, null);
        }
        public void start_async(string addr, int timeout, Action<PingReply, object> callback, object data)
        {
            Thread ping_thread = new Thread(() => ping_process(addr, timeout, callback, data));
            ping_thread.IsBackground = true;
            ping_thread.Start();
        }
        void ping_process(string addr, int timeout, Action<PingReply, object> callback, object data)
        {
            try
            {
                Ping pingSender = new Ping();
                PingReply reply = pingSender.Send(addr, timeout);
                callback(reply, data);
                pingSender.Dispose();
            }
            catch (Exception e)
            {
                log("!E: " + e.ToString());
                callback(null, data);
            }
        }
    }
}
