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
        bool ping_taskbar = false;
        rping ping;
        string ping_logpath = "";
        Hashtable ping_opts;
        void init_form_ping()
        {
            pinglog = new rlog(lb_ping);
            bar = new rtaskbar(pinglog.write, this.Handle);
            recq = new Queue();
            ping = new rping(ping_log_func);
        }
        void ping_log_func(string msg)
        {
            pinglog.write(msg);
        }
        void ping_cb(PingReply reply, object data)
        {
            if (reply == null)
            {
                this.Text = "Rabbit";
                cfg.set("ping_btn", Language.trans("开始"));
            }
            else display_taskbar(reply.Status == IPStatus.Success);
            text_pingstat.Text = ping.ToString();
        }
        void ping_parse_args()
        {
            ping_addr = ((TextBox)texthash["ping_addr"]).Text;
            ping_opts = ropt.parse_opts(text_pingopt.Text);
            if (ping_opts.ContainsKey("taskbar")) bool.TryParse((string)ping_opts["taskbar"], out ping_taskbar);
            if (ping_opts.ContainsKey("log")) ping_logpath = (string)ping_opts["log"];
            if (!string.IsNullOrEmpty(ping_logpath)) pinglog.savefile(ping_logpath);
        }
        void ping_click(object sender, EventArgs evt)
        {
            try
            {
                if (cfg.getstr("ping_btn") == Language.trans("开始"))
                {
                    this.Text = ((TextBox)texthash["ping_addr"]).Text;
                    cfg.set("ping_btn",Language.trans("停止"));
                    pinglog.clear();
                    ping_parse_args();
                    recq.Clear();
                    ping.start(ping_addr, ping_opts, ping_cb, null);
                }
                else
                {
                    ping.stop();
                    this.Text = "Rabbit";
                    cfg.set("ping_btn", Language.trans("开始"));
                }
            }
            catch (Exception) { }
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
