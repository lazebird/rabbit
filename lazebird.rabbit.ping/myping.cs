using System;
using System.Net.NetworkInformation;
using System.Threading;

namespace lazebird.rabbit.ping
{
    public class myping
    {
        Action<string> log;
        int reqid = 0;

        public myping(Action<string> log)
        {
            this.log = log;
        }
        public void start(string addr, int timeout, Action<int, PingReply> callback)
        {
            ping_process(++reqid, addr, timeout, callback);
        }
        public int start_async(string addr, int timeout, Action<int, PingReply> callback)
        {
            int id = ++reqid;
            Thread ping_thread = new Thread(() => ping_process(id, addr, timeout, callback));
            ping_thread.IsBackground = true;
            ping_thread.Start();
            return id;
        }
        private void ping_process(int id, string addr, int timeout, Action<int, PingReply> callback)
        {
            Ping pingSender = new Ping();
            PingReply reply;
            try
            {
                reply = pingSender.Send(addr, timeout);
                callback(id, reply);
                pingSender.Dispose();
            }
            catch (Exception e)
            {
                log("Error: " + e.Message);
                callback(id, null);
            }
        }
    }
}
