using lazebird.rabbit.common;
using lazebird.rabbit.ping;
using System;
using System.Collections;
using System.Drawing;
using System.Net;
using System.Net.NetworkInformation;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        string scan_startip;
        int scan_lastbyte;
        int scan_interval = 1000;
        int scan_count = 1;
        bool scan_stoponloss = false;
        Hashtable scansshash;
        int scan_start_stat;
        int scan_stop_stat;
        void init_form_scan()
        {
            btn_scan.Click += new EventHandler(scan_click);
            fp_scan.AutoScroll = true;
            scansshash = new Hashtable();
        }
        void scan_log_func(string msg)
        {
        }
        bool scan_hideunreachable;
        void scan_parse_args()
        {
            scan_startip = ((TextBox)texthash["scan_ipstart"]).Text;
            int.TryParse(((TextBox)texthash["scan_ipend"]).Text, out scan_lastbyte);
            Hashtable opts = ropt.parse_opts(text_scanopt.Text);
            if (opts.ContainsKey("interval")) int.TryParse((string)opts["interval"], out scan_interval);
            if (opts.ContainsKey("count")) int.TryParse((string)opts["count"], out scan_count);
            if (opts.ContainsKey("stoponloss")) bool.TryParse((string)opts["stoponloss"], out scan_stoponloss);
            if (opts.ContainsKey("hideunreachable")) bool.TryParse((string)opts["hideunreachable"], out scan_hideunreachable);
        }
        object scan_lock = new object();
        void scan_reply(PingReply reply, object data)
        {
            Label lb = (Label)data;
            if (reply == null)
            {
                scansshash.Remove(lb);
                lock (scan_lock) if (++scan_stop_stat == scan_start_stat) btn_scan.Text = "Start";
                return;
            }
            lb.BackColor = (reply.Status == IPStatus.Success) ? Color.Green : Color.FromArgb(64, 64, 64);
            if (scan_hideunreachable) lb.Visible = (reply.Status == IPStatus.Success);
        }
        void start_scan()
        {
            try
            {
                scan_parse_args();
                Byte[] ipbytes = IPAddress.Parse(scan_startip).GetAddressBytes();
                scan_start_stat = scan_stop_stat = 0;
                for (int i = ipbytes[3]; i <= scan_lastbyte && i < 255; i++)
                {
                    scan_start_stat++;
                    Label lb = new Label();
                    lb.Text = i.ToString();
                    lb.Width = lb.Height = 28;
                    lb.TextAlign = ContentAlignment.MiddleCenter;
                    lb.BackColor = Color.FromArgb(64, 64, 64);
                    lb.ForeColor = Color.White;
                    fp_scan.Controls.Add(lb);
                    ipbytes[3] = (Byte)i;
                    rping s = new rping(scan_log_func, (new IPAddress(ipbytes)).ToString(), scan_interval, scan_count, scan_stoponloss);
                    s.start(scan_reply, lb);
                    scansshash.Add(lb, s);
                }
            }
            catch (Exception) { }
        }
        void stop_scan()
        {
            foreach (Label lb in scansshash.Keys)
            {
                ((rping)scansshash[lb]).stop();
            }
        }
        void clear_scan()
        {
            fp_scan.Controls.Clear();
            foreach (Label lb in scansshash.Keys)
            {
                lb.Dispose();
            }
            scansshash.Clear();
        }
        void scan_click(object sender, EventArgs e)
        {
            if (btn_scan.Text == "Start")
            {
                btn_scan.Text = "Stop";
                clear_scan();
                start_scan();
            }
            else
            {
                btn_scan.Text = "Start";
                stop_scan();
            }
            saveconf();
        }
    }
}
