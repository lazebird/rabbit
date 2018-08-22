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
        Queue recq;
        rtaskbar bar;
        string ping_addr;
        int ping_interval = 1000;
        int ping_count = -1;
        bool ping_stoponloss = false;
        bool ping_taskbar = false;
        rping ping;
        string ping_logpath = "";
        void init_form_ping()
        {
            pinglog = new rlog(ping_output);
            btn_ping.Click += new EventHandler(ping_click);
            bar = new rtaskbar(pinglog.write);
            recq = new Queue();
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
            else display_taskbar(reply.Status == IPStatus.Success);
            if (ping != null) text_pingstat.Text = ping.ToString();
        }
        void ping_parse_args()
        {
            ping_addr = ((TextBox)texthash["ping_addr"]).Text;
            Hashtable opts = ropt.parse_opts(text_pingopt.Text);
            if (opts.ContainsKey("interval")) int.TryParse((string)opts["interval"], out ping_interval);
            if (opts.ContainsKey("count")) int.TryParse((string)opts["count"], out ping_count);
            if (opts.ContainsKey("stoponloss")) bool.TryParse((string)opts["stoponloss"], out ping_stoponloss);
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
                recq.Clear();
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
        void display_taskbar(bool state)
        {
            if (ping_taskbar) bar.reset();
            recq.Enqueue(state);
            while (recq.Count > 5) recq.Dequeue();
            int success = 0;
            int failure = 0;
            int tmp;
            foreach (bool s in recq) tmp = (s ? (success++) : (failure++));
            if (failure > 0) bar.set(4, failure, 5); // TaskbarProgressBarState.Error
            else bar.set(2, success, 5); // TaskbarProgressBarState.Normal
        }
    }
}
