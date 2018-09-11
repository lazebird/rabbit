﻿using lazebird.rabbit.common;
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
        rpanel scan_panel;
        string scan_startip;
        int scan_lastbyte;
        Hashtable scan_opts;
        Hashtable scansshash;
        void init_form_scan()
        {
            scan_panel = new rpanel(fp_scan, 28);
            btn_scan.Click += new EventHandler(scan_click);
            scansshash = Hashtable.Synchronized(new Hashtable());
        }
        void scan_log_func(string msg)
        {
        }
        bool scan_hideunreachable;
        void scan_parse_args()
        {
            scan_startip = ((TextBox)texthash["scan_ipstart"]).Text;
            int.TryParse(((TextBox)texthash["scan_ipend"]).Text, out scan_lastbyte);
            scan_opts = ropt.parse_opts(text_scanopt.Text);
            if (scan_opts.ContainsKey("hideunreachable")) bool.TryParse((string)scan_opts["hideunreachable"], out scan_hideunreachable);
        }
        void scan_reply(PingReply reply, object data)
        {
            TextBox tb = (TextBox)data;
            if (reply == null)
            {
                scansshash.Remove(tb);
                if (scansshash.Count == 0) btn_scan.Text = "Start";
                return;
            }
            if (reply.Status == IPStatus.Success) tb.BackColor = Color.Green;
            if (scan_hideunreachable) tb.Visible = (reply.Status == IPStatus.Success);
        }
        void start_scan()
        {
            try
            {
                scan_parse_args();
                Byte[] ipbytes = IPAddress.Parse(scan_startip).GetAddressBytes();
                for (int i = ipbytes[3]; i <= scan_lastbyte && i < 255; i++)
                {
                    TextBox tb = scan_panel.add(i.ToString(), null, null);
                    tb.Width = tb.Height = 28;
                    tb.TextAlign = HorizontalAlignment.Center;
                    ipbytes[3] = (Byte)i;
                    rping s = new rping(scan_log_func, (new IPAddress(ipbytes)).ToString(), scan_opts);
                    s.start(scan_reply, tb);
                    scansshash.Add(tb, s);
                }
            }
            catch (Exception) { }
        }
        void stop_scan()
        {
            foreach (TextBox tb in scansshash.Keys)
            {
                ((rping)scansshash[tb]).stop();
            }
        }
        void scan_click(object sender, EventArgs e)
        {
            if (btn_scan.Text == "Start")
            {
                btn_scan.Text = "Stop";
                scan_panel.clear();
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
