using System;
using System.Net.NetworkInformation;
using System.Threading;

namespace lazebird.rabbit.ping
{
    public class rping
    {
        Action<string> log;
        string addr;
        int interval;
        int count;
        bool stoponloss;
        bool runflag;
        DateTime opertm;
        int txcnt, rxcnt, losscnt;
        int mintm, maxtm, totaltm;

        public rping(Action<string> log, string addr)
        {
            this.log = log;
            this.addr = addr;
            interval = 1000;
            count = -1;
            stoponloss = false;
        }
        public rping(Action<string> log, string addr, int interval, int count, bool stoponloss) : this(log, addr)
        {
            this.interval = interval;
            this.count = count;
            this.stoponloss = stoponloss;
        }
        void ping_task(Action<PingReply, object> callback, object data)
        {
            Ping pingSender = new Ping();
            try
            {
                opertm = DateTime.Now;
                runflag = true;
                txcnt = rxcnt = losscnt = 0;
                mintm = 0xfffffff;
                maxtm = totaltm = 0;
                PingReply reply;
                while (runflag && (count < 0 || count-- > 0))
                {
                    reply = pingSender.Send(addr, interval);
                    callback?.Invoke(reply, data);
                    txcnt++;
                    progress_display(reply);
                    if (stoponloss && reply.Status != IPStatus.Success) break;
                }
            }
            catch (Exception e)
            {
                log("!E: " + e.ToString());
            }
            session_display();
            opertm = DateTime.Now;
            callback?.Invoke(null, data);
            pingSender.Dispose();
        }
        public void start(Action<PingReply, object> callback, object data)
        {
            Thread t = new Thread(() => ping_task(callback, data));
            t.IsBackground = true;
            t.Start();
        }
        public void stop()
        {
            runflag = false;
        }
        public override string ToString()
        {
            float avgtm = (float)totaltm / Math.Max(1, rxcnt);
            return opertm.ToString() + " Tx " + txcnt + " Rx " + rxcnt + " Loss " + losscnt
                 + " Min " + mintm + " Max " + maxtm + " Avg " + avgtm;
        }
        void progress_display(PingReply reply)
        {
            if (reply.Status == IPStatus.Success)
            {
                rxcnt++;
                mintm = Math.Min(mintm, (int)reply.RoundtripTime);
                maxtm = Math.Max(maxtm, (int)reply.RoundtripTime);
                totaltm += (int)reply.RoundtripTime;
                log("来自 " + reply.Address + " 的回复: 字节=" + reply.Buffer.Length + " 毫秒=" + reply.RoundtripTime + " TTL=" + reply.Options.Ttl);
                if (interval > (int)reply.RoundtripTime) Thread.Sleep(interval - (int)reply.RoundtripTime);
            }
            else
            {
                losscnt++;
                log("请求超时。");
            }
        }
        void session_display()
        {
            float lossper = (float)losscnt / Math.Max(1, txcnt) * 100;
            float avgtm = (float)totaltm / Math.Max(1, rxcnt);
            log("发送 = " + txcnt + "，接收 = " + rxcnt + "，丢失 = " + losscnt + " (" + lossper.ToString() + " % 丢失)");
            log("最短 = " + mintm + "毫秒，最长 = " + maxtm + "毫秒，平均 = " + avgtm.ToString() + "毫秒");
        }
    }
}
