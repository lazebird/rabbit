using rabbit.common;
using System;
using System.Net.NetworkInformation;
using System.Threading;

namespace rabbit.ping
{
    class myping
    {
        string addr;
        int timeout, times;
        Thread ping_thread;
        int txcnt, rxcnt, losscnt;
        int mintm, maxtm, totaltm;
        int[] records;
        int recordidx;
        mytaskbar bar;
        mylog l;
        private Action ping_stop_cb;
        Thread bar_test_thread;

        public myping(mylog l, Action ping_stop_cb)
        {
            this.l = l;
            this.ping_stop_cb = ping_stop_cb;
            bar = new mytaskbar();
            records = new int[5];
            bar_test_thread = new Thread(bar_test);
            bar_test_thread.IsBackground = true;
            bar_test_thread.Start();
        }
        private void bar_test()
        {
            Thread.Sleep(200);
            if (!bar.set(0, 0, 0))
            {
                l.write("Error: " + bar.strerr());
            }
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
            mintm = 0xfffffff;
            maxtm = totaltm = 0;
            while (times < 0 || times-- > 0)
            {
                try
                {
                    reply = pingSender.Send(addr, timeout);
                }
                catch (Exception e)
                {
                    l.write("Error: " + e.Message);
                    break;
                }
                txcnt++;
                if (reply.Status == IPStatus.Success)
                {
                    rxcnt++;
                    display_taskbar(1);
                    l.write("来自 " + reply.Address + " 的回复: 字节=" + reply.Buffer.Length + " 毫秒=" + reply.RoundtripTime + " TTL=" + reply.Options.Ttl);
                    if (timeout > (int)reply.RoundtripTime)
                    {
                        Thread.Sleep(timeout - (int)reply.RoundtripTime);
                    }
                    mintm = Math.Min(mintm, (int)reply.RoundtripTime);
                    maxtm = Math.Max(maxtm, (int)reply.RoundtripTime);
                    totaltm += (int)reply.RoundtripTime;
                }
                else
                {
                    losscnt++;
                    display_taskbar(0);
                    l.write("请求超时。");
                }
            }
            display_statistics();
            ping_stop_cb();
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
                bar.set(4, recordidx - count, 5); // TaskbarProgressBarState.Error
            }
            else
            {
                bar.set(2, count, 5); // TaskbarProgressBarState.Normal
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
            l.write("发送 = " + txcnt + "，接收 = " + rxcnt + "，丢失 = " + losscnt + " (" + lossper.ToString() + " % 丢失)");
            l.write("最短 = " + mintm + "毫秒，最长 = " + maxtm + "毫秒，平均 = " + avgtm.ToString() + "毫秒");
        }
    }
}
