using System;
using System.Net.NetworkInformation;
using System.Threading;

namespace ping
{
    class myping
    {
        static myping instance;
        static Form1 form;
        static string addr;
        static int timeout, times;
        static Thread ping_thread;
        static int txcnt, rxcnt, losscnt;
        static int mintm, maxtm, totaltm;
        static int[] records;
        static int recordidx;
        public static myping getinstance()
        {
            if (instance == null)
            {
                instance = new myping();
                records = new int[5];
            }
            return instance;
        }
        public void setform(Form1 f)
        {
            form = f;
        }
        public void start(string addr1, int timeout1, int times1)
        {
            addr = addr1;
            timeout = timeout1;
            times = times1;
            recordidx = 0;
            ping_thread = new Thread(ping_process);
            ping_thread.IsBackground = true;
            ping_thread.Start();
        }
        public void stop()
        {
            times = 0;
        }
        private void ping_process()
        {
            Ping pingSender = new Ping();
            PingReply reply;
            txcnt = rxcnt = losscnt = 0;
            mintm = maxtm = totaltm = 0;
            while (times < 0 || times-- > 0)
            {
                try
                {
                    reply = pingSender.Send(addr, timeout);
                }
                catch (Exception e)
                {
                    mylog.log("Error: " + e.Message);
                    form.setvalue("btn", "开始");
                    return;
                }
                txcnt++;
                if (reply.Status == IPStatus.Success)
                {
                    rxcnt++;
                    display_taskbar(1);
                    mylog.log("来自 " + reply.Address + " 的回复: 字节=" + reply.Buffer.Length + " 毫秒=" + reply.RoundtripTime + " TTL=" + reply.Options.Ttl);
                    if (timeout > (int)reply.RoundtripTime)
                    {
                        Thread.Sleep(timeout - (int)reply.RoundtripTime);
                    }

                    if (mintm == 0)
                    {
                        mintm = (int)reply.RoundtripTime;
                    }
                    else
                    {
                        mintm = Math.Min(mintm, (int)reply.RoundtripTime);
                    }
                    if (maxtm == 0)
                    {
                        maxtm = (int)reply.RoundtripTime;
                    }
                    else
                    {
                        maxtm = Math.Max(maxtm, (int)reply.RoundtripTime);
                    }
                    totaltm += (int)reply.RoundtripTime;
                }
                else
                {
                    losscnt++;
                    display_taskbar(0);
                    mylog.log("请求超时。");
                }
            }
            display_statistics();
            form.setvalue("btn", "开始");
        }
        private void display_taskbar(int rx)
        {
            if (recordidx < 5)
            {
                records[recordidx++] = rx;
            }
            else
            {
                for (int i = 0; i < 4; i++)
                {
                    records[i] = records[i + 1];
                }
                records[4] = rx;
            }
            int count = 0;
            for (int i = 0; i < recordidx; i++)
            {
                count += records[i];
            }
            if (count != recordidx)
            {
                mytaskbar.set(4, recordidx - count, 5); // TaskbarProgressBarState.Error
            }
            else
            {
                mytaskbar.set(2, count, 5); // TaskbarProgressBarState.Normal
            }
        }
        private void display_statistics()
        {
            float lossper, avgtm;
            if (txcnt != 0)
            {
                lossper = (float)losscnt / txcnt * 100;
            }
            else
            {
                lossper = 0;
            }
            if (rxcnt != 0)
            {
                avgtm = (float)totaltm / rxcnt;
            }
            else
            {
                avgtm = 0;
            }
            mylog.log("发送 = " + txcnt + "，接收 = " + rxcnt + "，丢失 = " + losscnt + " (" + lossper.ToString() + " % 丢失)");
            mylog.log("最短 = " + mintm + "毫秒，最长 = " + maxtm + "毫秒，平均 = " + avgtm.ToString() + "毫秒");
        }
    }
}
