﻿using System;
using System.Collections;
using System.Drawing;
using System.Net;
using System.Net.Sockets;
using System.Windows.Forms;

namespace lazebird.rabbit.chat
{
    public partial class rntfform : Form
    {
        Action<string> log;
        string luser;
        string ip;
        int port;
        Hashtable mhash;
        public rntfform(Action<string> log, string luser, string ip, int port)
        {
            InitializeComponent();
            this.log = log;
            this.luser = luser;
            this.ip = ip;
            this.port = port;
            mhash = new Hashtable();
            init_form();
        }
        void init_form()
        {
            this.Resize += msg_resize;
            rtb_chat.KeyUp += msg_key_press;
        }
        void send_notification(int port, string msg)
        {
            UdpClient uc_ntf = new UdpClient();
            pkt p = new ntf_pkt(luser, msg);
            IPEndPoint r = new IPEndPoint(IPAddress.Parse(ip), port);
            byte[] buf = p.pack();
            uc_ntf.SendAsync(buf, buf.Length, r);
            uc_ntf.Dispose();
        }
        void msg_key_press(object sender, KeyEventArgs e)
        {
            RichTextBox tb = (RichTextBox)sender;
            if (e.KeyData == (Keys.Enter))
            {
                string s = tb.Text.Trim();
                if (string.IsNullOrEmpty(s)) return;
                send_notification(port, s);
                show_msg(s);
                tb.Clear();
            }
            else if (e.KeyData == (Keys.Control | Keys.S))
            {
            }
            else if (e.KeyData == Keys.Escape)
            {
                Close();
            }
        }
        int pa_chat_gap = 30;
        int tb_chat_gap = 12;
        void msg_resize(object sender, EventArgs e)
        {
            pa_chat.Width = Width - pa_chat_gap;
            foreach (RichTextBox tb in mhash.Values) tb.Width = pa_chat.Width - tb_chat_gap;
            log("I: form width " + Width + " fp width " + pa_chat.Width + " tb width " + (pa_chat.Width - tb_chat_gap));
        }
        void rtb_ContentsResized(object sender, ContentsResizedEventArgs e)
        {
            ((RichTextBox)sender).Height = e.NewRectangle.Height + 5;
        }
        void show_msg(string s)
        {
            message m = new message(luser, s);
            RichTextBox rtb = new RichTextBox();
            rtb.Width = pa_chat.Width - tb_chat_gap;
            rtb.WordWrap = true;
            rtb.ContentsResized += rtb_ContentsResized;
            rtb.Font = new Font(rtb.Font.FontFamily, 12, rtb.Font.Style);
            rtb.BorderStyle = BorderStyle.None;
            rtb.BackColor = pa_chat.BackColor;
            rtb.ForeColor = Color.White;
            rtb.Dock = DockStyle.Top;    // auto add behind pevious tb
            pa_chat.Controls.Add(rtb);
            pa_chat.Controls.SetChildIndex(rtb, 0);
            pa_chat.ScrollControlIntoView(rtb);
            mhash.Add(m, rtb);
            rtb.Text = m.ToString();
        }
        void del_msg(message m)
        {
            RichTextBox tb;
            if (mhash.ContainsKey(m))
            {
                tb = (RichTextBox)mhash[m];
                pa_chat.Controls.Remove(tb);
                mhash.Remove(m);
            }
        }
    }
}
