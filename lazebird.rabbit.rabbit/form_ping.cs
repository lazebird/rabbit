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
            btn_ping.Click += new EventHandler(ping_click);
            bar = new rtaskbar(pinglog.write, this.Handle);
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
            ping_opts = ropt.parse_opts(text_pingopt.Text);
            if (ping_opts.ContainsKey("taskbar")) bool.TryParse((string)ping_opts["taskbar"], out ping_taskbar);
            if (ping_opts.ContainsKey("log")) ping_logpath = (string)ping_opts["log"];
            if (!string.IsNullOrEmpty(ping_logpath)) pinglog.savefile(ping_logpath);
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
                ping = new rping(ping_log_func, ping_addr, ping_opts);
                ping.start(ping_cb, null);
            }
            else
            {
                if (ping != null) ping.stop();
                ping = null;
                ((Form)formhash["form"]).Text = "Rabbit";
                ((Button)btnhash["ping_btn"]).Text = Language.trans("开始");
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
