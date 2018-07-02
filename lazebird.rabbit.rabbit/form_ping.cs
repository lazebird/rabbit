using lazebird.rabbit.common;
using lazebird.rabbit.ping;
using System;
using System.Net.NetworkInformation;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rping ping;
        mylog pinglog;
        void init_form_ping()
        {
            pinglog = new mylog(ping_output);
            ping = new rping(ping_log_func);
            btn_ping.Click += new EventHandler(ping_click);
            btn_ping_log.Click += new EventHandler(ping_log_click);
            bar = new mytaskbar();
            records = new int[5];
        }
        int txcnt, rxcnt, losscnt;
        int mintm, maxtm, totaltm;
        int[] records;
        int recordidx;
        mytaskbar bar;
        void bar_test()
        {
            Thread.Sleep(500);
            if (!bar.set(0, 0, 0))
            {
                pinglog.write("Error: " + bar.strerr());
            }
        }
        void ping_log_func(string msg)
        {
            pinglog.write(msg);
        }
        long RoundtripTime;
        void ping_cb(int id, PingReply reply)
        {
            if (reply == null)  // exception
            {
                RoundtripTime = 0;
                stop_ping();
            }
            else if (reply.Status == IPStatus.Success)
            {
                rxcnt++;
                display_taskbar(1);
                mintm = Math.Min(mintm, (int)reply.RoundtripTime);
                maxtm = Math.Max(maxtm, (int)reply.RoundtripTime);
                totaltm += (int)reply.RoundtripTime;
                RoundtripTime = reply.RoundtripTime;
                pinglog.write("来自 " + reply.Address + " 的回复: 字节=" + reply.Buffer.Length + " 毫秒=" + reply.RoundtripTime + " TTL=" + reply.Options.Ttl);
            }
            else
            {
                losscnt++;
                display_taskbar(0);
                RoundtripTime = int.Parse(((TextBox)texthash["ping_timeout"]).Text);
                pinglog.write("请求超时。");
            }
        }
        bool stop_unset;
        void start_ping()
        {
            string addr = ((TextBox)texthash["ping_addr"]).Text;
            int timeout = int.Parse(((TextBox)texthash["ping_timeout"]).Text);
            int count = int.Parse(((TextBox)texthash["ping_times"]).Text);
            ((Form)formhash["form"]).Text = ((TextBox)texthash["ping_addr"]).Text;
            ((Button)btnhash["ping_btn"]).Text = Language.trans("停止");
            stop_unset = true;
            recordidx = 0;
            txcnt = rxcnt = losscnt = 0;
            mintm = 0xfffffff;
            maxtm = totaltm = 0;
            while (stop_unset && (count < 0 || count-- > 0))
            {
                ping.start(addr, timeout, ping_cb);
                txcnt++;
                if (timeout > (int)RoundtripTime)
                {
                    Thread.Sleep(timeout - (int)RoundtripTime);
                }
            }
            display_statistics();
            stop_ping();
        }
        void stop_ping()
        {
            stop_unset = false;
            ((Form)formhash["form"]).Text = "Rabbit";
            ((Button)btnhash["ping_btn"]).Text = Language.trans("开始");
        }
        void ping_click(object sender, EventArgs evt)
        {
            if (((Button)btnhash["ping_btn"]).Text == Language.trans("开始"))
            {
                pinglog.setfile(((TextBox)texthash["ping_logfile"]).Text);
                pinglog.clear();
                Thread th = new Thread(start_ping);
                th.IsBackground = true;
                th.Start();
            }
            else
            {
                stop_ping();
            }
            saveconf(); // save empty config to restore default config
        }
        void ping_log_click(object sender, EventArgs e)
        {
            SaveFileDialog fd = new SaveFileDialog();
            //filename.InitialDirectory = Application.StartupPath;
            fd.Filter = "文本文件 (*.txt)|*.txt|All files (*.*)|*.*";
            fd.RestoreDirectory = true;
            fd.CreatePrompt = true;
            fd.OverwritePrompt = true;
            fd.InitialDirectory = Environment.CurrentDirectory;
            if (fd.ShowDialog() == DialogResult.OK)
            {
                ((TextBox)texthash["ping_logfile"]).Text = fd.FileName;
            }
        }
        void display_taskbar(int rx)
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
        void display_statistics()
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
            pinglog.write("发送 = " + txcnt + "，接收 = " + rxcnt + "，丢失 = " + losscnt + " (" + lossper.ToString() + " % 丢失)");
            pinglog.write("最短 = " + mintm + "毫秒，最长 = " + maxtm + "毫秒，平均 = " + avgtm.ToString() + "毫秒");
        }
    }
}
