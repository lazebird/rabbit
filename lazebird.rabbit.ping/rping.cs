using System;
using System.Collections;
using System.Collections.Generic;
using System.Net.NetworkInformation;
using System.Threading;

namespace lazebird.rabbit.ping
{
    public class rping
    {
        Action<string> log;
        string addr;
        Dictionary<string, int> idic;
        Dictionary<string, bool> bdic;
        Thread t;
        bool run_flag;
        DateTime opertm;
        int txcnt, rxcnt, losscnt;
        int mintm, maxtm, totaltm;
        public rping(Action<string> log)
        {
            this.log = log;
        }
        void init_args(Hashtable opts)
        {
            idic = new Dictionary<string, int>();
            bdic = new Dictionary<string, bool>();
            idic.Add("interval", 1000); // ms
            idic.Add("count", -1);
            bdic.Add("stoponloss", false);
            foreach (string key in opts.Keys)
            {
                if (idic.ContainsKey(key)) idic[key] = int.Parse((string)opts[key]);
                else if (bdic.ContainsKey(key)) bdic[key] = bool.Parse((string)opts[key]);
            }
        }

        void ping_handler(Action<PingReply, object> callback, object data)
        {
            Ping pingSender = new Ping();
            opertm = DateTime.Now;
            run_flag = true;
            txcnt = rxcnt = losscnt = 0;
            mintm = 0xfffffff;
            maxtm = totaltm = 0;
            PingReply reply;
            while (run_flag && (idic["count"] < 0 || idic["count"]-- > 0))
            {
                reply = null;
                try
                {
                    reply = pingSender.Send(addr, idic["interval"]);
                    callback?.Invoke(reply, data);
                    txcnt++;
                    progress_display(reply);
                    if (bdic["stoponloss"] && reply.Status != IPStatus.Success) break;
                }
                catch (Exception e)
                {
                    log("!E: " + e.ToString());
                }
                if (reply == null) Thread.Sleep(idic["interval"]);
                else if (reply.Status == IPStatus.Success && idic["interval"] > (int)reply.RoundtripTime) Thread.Sleep(idic["interval"] - (int)reply.RoundtripTime);
            }
            session_display();
            opertm = DateTime.Now;
            callback?.Invoke(null, data);
            pingSender.Dispose();
        }
        void ping_task(Action<PingReply, object> callback, object data)
        {
            try
            {
                ping_handler(callback, data);
            }
            catch (Exception e)
            {
                log("!E: " + e.ToString());
            }
        }
        public void start(string addr, Hashtable opts, Action<PingReply, object> callback, object data)
        {
            this.addr = addr;
            init_args(opts);
            if (t != null) return;
            t = new Thread(() => ping_task(callback, data));
            t.IsBackground = true;
            t.Start();
        }
        public void stop()
        {
            run_flag = false;
            t = null;
        }
        public override string ToString()
        {
            float avgtm = (float)totaltm / Math.Max(1, rxcnt);
            return opertm.ToString("yyyy/M/d HH:mm:ss") + " Tx " + txcnt + " Rx " + rxcnt + " Loss " + losscnt
                 + " Min " + mintm + " Max " + maxtm + " Avg " + avgtm;
        }
        void progress_display(PingReply reply)
        {
            if (reply.Status != IPStatus.Success)
            {
                losscnt++;
                log("请求超时。");
                return;
            }
            rxcnt++;
            mintm = Math.Min(mintm, (int)reply.RoundtripTime);
            maxtm = Math.Max(maxtm, (int)reply.RoundtripTime);
            totaltm += (int)reply.RoundtripTime;
            log("来自 " + reply.Address + " 的回复: 字节=" + reply.Buffer.Length + " 毫秒=" + reply.RoundtripTime + ((reply.Options != null) ? (" TTL=" + reply.Options.Ttl) : ""));
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
