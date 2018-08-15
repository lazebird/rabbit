using lazebird.rabbit.common;
using lazebird.rabbit.ping;
using System;
using System.Collections;
using System.Net.NetworkInformation;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rlog pinglog;
        int[] records;
        int recordidx;
        rtaskbar bar;
        string ping_addr;
        int ping_interval = 1000;
        int ping_count = -1;
        bool ping_stoponloss = false;
        rping ping;
        string ping_logpath = "";
        void init_form_ping()
        {
            pinglog = new rlog(ping_output);
            btn_ping.Click += new EventHandler(ping_click);
            bar = new rtaskbar(pinglog.write);
            records = new int[5];
        }
        void ping_log_func(string msg)
        {
            pinglog.write(msg);
        }
        void ping_cb(PingReply reply, object data)
        {
            if (reply == null)
            {
                ((Form)formhash["form"]).Text = "Rabbit";
                ((Button)btnhash["ping_btn"]).Text = Language.trans("开始");
            }
            else display_taskbar(reply.Status == IPStatus.Success ? 1 : 0);
            if (ping != null) text_pingstat.Text = ping.ToString();
        }
        void ping_parse_args()
        {
            ping_addr = ((TextBox)texthash["ping_addr"]).Text;
            Hashtable opts = ropt.parse_opts(text_pingopt.Text);
            if (opts.ContainsKey("interval")) ping_interval = int.Parse((string)opts["interval"]);
            if (opts.ContainsKey("count")) ping_count = int.Parse((string)opts["count"]);
            if (opts.ContainsKey("stoponloss")) ping_stoponloss = (string)opts["stoponloss"] == "true";
            if (opts.ContainsKey("log")) ping_logpath = (string)opts["log"];
            if (!string.IsNullOrEmpty(ping_logpath)) pinglog.setfile(ping_logpath);
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
                ping = new rping(ping_log_func, ping_addr, ping_interval, ping_count, ping_stoponloss);
                ping.start(ping_cb, null);
            }
            else
            {
                if (ping != null) ping.stop();
                ping = null;
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
    }
}
