﻿using lazebird.rabbit.common;
using lazebird.rabbit.ping;
using System;
using System.Collections;
using System.Net.NetworkInformation;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rping ping;
        rlog pinglog;
        void init_form_ping()
        {
            pinglog = new rlog(ping_output);
            ping = new rping(ping_log_func);
            btn_ping.Click += new EventHandler(ping_click);
            bar = new rtaskbar(pinglog.write);
            records = new int[5];
        }
        int txcnt, rxcnt, losscnt;
        int mintm, maxtm, totaltm;
        int[] records;
        int recordidx;
        rtaskbar bar;
        void ping_log_func(string msg)
        {
            pinglog.write(msg);
        }
        long RoundtripTime;
        void ping_cb(PingReply reply, object data)
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
                RoundtripTime = ping_interval;
                pinglog.write("请求超时。");
            }
        }
        bool stop_unset;
        string ping_addr;
        int ping_interval = 1000;
        int ping_count = -1;
        string ping_logpath = "";
        void ping_parse_args()
        {
            ping_addr = ((TextBox)texthash["ping_addr"]).Text;
            Hashtable opts = parse_opts(text_pingopt.Text);
            if (opts.ContainsKey("interval")) ping_interval = int.Parse((string)opts["interval"]);
            if (opts.ContainsKey("count")) ping_count = int.Parse((string)opts["count"]);
            if (opts.ContainsKey("log")) ping_logpath = (string)opts["log"];
            if (!string.IsNullOrEmpty(ping_logpath)) pinglog.setfile(ping_logpath);
        }
        void start_ping()
        {
            try
            {
                stop_unset = true;
                recordidx = 0;
                txcnt = rxcnt = losscnt = 0;
                mintm = 0xfffffff;
                maxtm = totaltm = 0;
                while (stop_unset && (ping_count < 0 || ping_count-- > 0))
                {
                    ping.start(ping_addr, ping_interval, ping_cb, null);
                    txcnt++;
                    if (ping_interval > (int)RoundtripTime)
                    {
                        Thread.Sleep(ping_interval - (int)RoundtripTime);
                    }
                }
                display_statistics();
                stop_ping();
            }
            catch (Exception e)
            {
                pinglog.write("!E: " + e.ToString());
            }
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
                ((Form)formhash["form"]).Text = ((TextBox)texthash["ping_addr"]).Text;
                ((Button)btnhash["ping_btn"]).Text = Language.trans("停止");
                pinglog.clear();
                ping_parse_args();
                bar.reset();
                Thread th = new Thread(start_ping);
                th.IsBackground = true;
                th.Start();
            }
            else
            {
                stop_ping();
            }
            saveconf();
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
