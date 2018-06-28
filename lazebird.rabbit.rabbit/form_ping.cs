using lazebird.rabbit.common;
using lazebird.rabbit.ping;
using System;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        myping ping;
        mylog pinglog;
        void init_form_ping()
        {
            pinglog = new mylog(ping_output);
            ping = new myping(pinglog, ping_stop_cb);
            btn_ping.Click += new EventHandler(ping_click);
            btn_ping_log.Click += new EventHandler(ping_log_click);
            bar = new mytaskbar();
            records = new int[5];
            bar_test_thread = new Thread(bar_test);
            bar_test_thread.IsBackground = true;
            bar_test_thread.Start();
        }
        int txcnt, rxcnt, losscnt;
        int mintm, maxtm, totaltm;
        int[] records;
        int recordidx;
        mytaskbar bar;
        Thread bar_test_thread;
        private void bar_test()
        {
            Thread.Sleep(500);
            if (!bar.set(0, 0, 0))
            {
                pinglog.write("Error: " + bar.strerr());
            }
        }
        private void ping_cb(int id, IPStatus Status, long RoundtripTime)
        {
            if (Status == IPStatus.Success)
            {
                rxcnt++;
                display_taskbar(1);
                //if (timeout > (int)RoundtripTime)
                //{
                //    Thread.Sleep(timeout - (int)RoundtripTime);
                //}
                mintm = Math.Min(mintm, (int)RoundtripTime);
                maxtm = Math.Max(maxtm, (int)RoundtripTime);
                totaltm += (int)RoundtripTime;
                log("来自 " + Address + " 的回复: 字节=" + Buffer.Length + " 毫秒=" + RoundtripTime + " TTL=" + Options.Ttl);
            }
            else
            {
                losscnt++;
                display_taskbar(0);
                log("请求超时。");
            }
        }
        private void start_ping(string addr, int timeout, int count)
        {
            recordidx = 0;
            txcnt = rxcnt = losscnt = 0;
            mintm = 0xfffffff;
            maxtm = totaltm = 0;
            while (count < 0 || count-- > 0)
            {
                ping.start(addr, timeout, ping_cb);
                txcnt++;
                Thread.Sleep(timeout);
            }
        }
        private void stop_ping()
        {
            display_statistics();
        }
        private void ping_click(object sender, EventArgs evt)
        {
            if (((Button)btnhash["ping_btn"]).Text == "开始")
            {
                try
                {
                    ((Form)formhash["form"]).Text = ((TextBox)texthash["ping_addr"]).Text;
                    pinglog.setfile(((TextBox)texthash["ping_logfile"]).Text);
                    pinglog.clear();
                    start_ping(((TextBox)texthash["ping_addr"]).Text, int.Parse(((TextBox)texthash["ping_timeout"]).Text), int.Parse(((TextBox)texthash["ping_times"]).Text));
                    ((Button)btnhash["ping_btn"]).Text = "停止";
                }
                catch (Exception e)
                {
                    pinglog.write("Error: " + e.Message);
                }
            }
            else
            {
                ping.stop();
            }
            saveconf(); // save empty config to restore default config
        }
        private void ping_log_click(object sender, EventArgs e)
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
            log("发送 = " + txcnt + "，接收 = " + rxcnt + "，丢失 = " + losscnt + " (" + lossper.ToString() + " % 丢失)");
            log("最短 = " + mintm + "毫秒，最长 = " + maxtm + "毫秒，平均 = " + avgtm.ToString() + "毫秒");
        }
    }
}
