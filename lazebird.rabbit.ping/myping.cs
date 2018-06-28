using lazebird.rabbit.common;
using System;
using System.Net.NetworkInformation;
using System.Threading;

namespace lazebird.rabbit.ping
{
    public class myping
    {
        Action<string> log;

        public myping(Action<string> log)
        {
            this.log = log;
        }
        public void start(string addr, int timeout, Action<int, IPStatus, long> callback)
        {
            Thread ping_thread = new Thread(() => ping_process(addr, timeout, callback));
            ping_thread.IsBackground = true;
            ping_thread.Start();
        }
        private void ping_process(string addr, int timeout, Action<int, IPStatus, long> callback)
        {
            Ping pingSender = new Ping();
            PingReply reply;
            try
            {
                reply = pingSender.Send(addr, timeout);
                callback(0, reply.Status, reply.RoundtripTime);
            }
            catch (Exception e)
            {
                log("Error: " + e.Message);
            }
        }
    }
}
