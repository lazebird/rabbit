using System;
using System.Windows.Forms;
using Microsoft.WindowsAPICodePack.Shell;
using Microsoft.WindowsAPICodePack.Taskbar;
using System.Net.NetworkInformation;
using System.Threading;
using System.IO;

namespace ping
{
    public partial class Form1 : Form
    {
        public Form1()
        {
            InitializeComponent();
            records = new int[5];
            this.AcceptButton = button1;
        }
        StreamWriter file;
        string addr;
        int state, timeout, times;
        Thread ping_thread;
        int txcnt, rxcnt, losscnt;
        int mintm, maxtm, totaltm;
        int[] records;
        int recordidx;
        int taskbar_support;

        private void taskbar_dll_detect_safe()
        {
            try
            {
                taskbar_dll_detect();
                taskbar_support = 1;
            }
            catch (Exception e)
            {
                logprint("Error: " + e.Message);
                taskbar_support = 0;
            }
        }
        private void taskbar_dll_detect()
        {
            TaskbarManager.Instance.SetProgressState(TaskbarProgressBarState.NoProgress);
        }
        private void display_taskbar_safe(int rx)
        {
            if (taskbar_support == 1)
            {
                display_taskbar(rx);
            }
        }
        private void display_taskbar(int rx)
        {
            if (!TaskbarManager.IsPlatformSupported)
            {
                return;
            }
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
                TaskbarManager.Instance.SetProgressState(TaskbarProgressBarState.Error);
                TaskbarManager.Instance.SetProgressValue(recordidx - count, 5);
            }
            else
            {
                TaskbarManager.Instance.SetProgressState(TaskbarProgressBarState.Normal);
                TaskbarManager.Instance.SetProgressValue(count, 5);
            }
        }

        private void parse_params()
        {
            try
            {
                addr = textBox1.Text;
                timeout = int.Parse(textBox2.Text);
                times = int.Parse(textBox3.Text);
            }
            catch (Exception e)
            {
                logprint("Error: " + e.Message);
            }
        }
        private void display_statistics()
        {
            state = 0;
            button1.Text = "开始";
            button1.Refresh();
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
            logprint("发送 = " + txcnt + "，接收 = " + rxcnt + "，丢失 = " + losscnt + " (" + lossper.ToString() + " % 丢失)");
            logprint("最短 = " + mintm + "毫秒，最长 = " + maxtm + "毫秒，平均 = " + avgtm.ToString() + "毫秒");
        }
        private void ping_process()
        {
            Ping pingSender = new Ping();
            PingReply reply;
            txcnt = rxcnt = losscnt = 0;
            mintm = maxtm = totaltm = 0;
            parse_params();
            if (addr == "")
            {
                state = 0;
                button1.Text = "停止";
                button1.Refresh();
                logprint("地址错误。");
                return;
            }
            while (times < 0 || times-- > 0)
            {
                try
                {

                    reply = pingSender.Send(addr, timeout);
                    txcnt++;
                    if (reply.Status == IPStatus.Success)
                    {
                        rxcnt++;
                        logprint("来自 " + reply.Address + " 的回复: 字节=" + reply.Buffer.Length + " 毫秒=" + reply.RoundtripTime + " TTL=" + reply.Options.Ttl);
                        display_taskbar_safe(1);
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
                        logprint("请求超时。");
                        display_taskbar_safe(0);
                    }
                }
                catch (Exception e)
                {
                    logprint("Error: " + e.Message);
                    return;
                }
            }
            display_statistics();
        }
        private void button1_Click(object sender, EventArgs e)
        {
            if (state == 0)
            {
                output.Items.Clear(); // clear old logs
                taskbar_dll_detect_safe();
                recordidx = 0;
                state = 1;
                button1.Text = "停止";
                button1.Refresh();
                // start
                CheckForIllegalCrossThreadCalls = false;
                ping_thread = new Thread(ping_process);
                ping_thread.IsBackground = true;
                ping_thread.Start();
            }
            else
            {
                // stop
                times = 0;
            }
        }

        private void button2_Click(object sender, EventArgs e)
        {
            SaveFileDialog filename = new SaveFileDialog();
            //filename.InitialDirectory = Application.StartupPath;
            filename.Filter = "文本文件 (*.txt)|*.txt|All files (*.*)|*.*";
            filename.RestoreDirectory = true;
            filename.CreatePrompt = true;
            filename.OverwritePrompt = true;
            if (filename.ShowDialog() == DialogResult.OK)
            {
                textBox4.Text = filename.FileName;
            }
        }
        private void logprint(string msg)
        {
            if (textBox4.Text != "")
            {
                file = new StreamWriter(textBox4.Text, true);
                file.WriteLine(msg);
                file.Close();
            }
            output.Items.Add(msg);
            output.TopIndex = output.Items.Count - (int)(output.Height / output.ItemHeight);
            output.Refresh();
        }
    }
}
