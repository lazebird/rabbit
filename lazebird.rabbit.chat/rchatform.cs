using System;
using System.Collections;
using System.Drawing;
using System.Net;
using System.Windows.Forms;

namespace lazebird.rabbit.chat
{
    public partial class rchatform : Form
    {
        Action<string> log;
        string luser;
        string ruser;
        IPEndPoint r;
        ss chatss;
        Hashtable mhash;
        public rchatform(Action<string> log, string luser, string ruser, IPEndPoint r)
        {
            InitializeComponent();
            this.log = log;
            this.luser = luser;
            this.ruser = ruser;
            this.r = r;
            Text = ruser + " " + r.ToString();
            chatss = new ss(log, luser, ruser, r);
            chatss.init_view_api(show_msg, del_msg);
            mhash = new Hashtable();
            init_form();
        }
        public rchatform(Action<string> log, string luser, pkt pkt, IPEndPoint r) : this(log, luser, pkt.user, r)
        {
            chatss.pkt_proc(pkt);
        }
        void init_form()
        {
            this.Resize += msg_resize;
            pa_chat.DoubleClick += msg_save_click;
            rtb_chat.KeyUp += msg_key_press;
        }
        void msg_save_click(object sender, EventArgs e)
        {
            SaveFileDialog fd = new SaveFileDialog();
            fd.Title = "Select a file to save";
            fd.Filter = "文本文件 (*.txt)|*.txt|All files (*.*)|*.*";
            fd.CreatePrompt = true;
            fd.OverwritePrompt = true;
            fd.ShowHelp = true;
            log("I: saving logs");
            if (fd.ShowDialog() == DialogResult.OK)
            {
                log("I: saving logs to " + fd.FileName);
                chatss.log2txt(fd.FileName);
            }
        }
        void msg_key_press(object sender, KeyEventArgs e)
        {
            RichTextBox tb = (RichTextBox)sender;
            if (e.KeyData == (Keys.Enter))
            {
                if (!string.IsNullOrEmpty(tb.Text.Trim())) chatss.write_msg(tb.Text.Trim());
                tb.Clear();
            }
            else if (e.KeyData == (Keys.Control | Keys.S))
            {
                msg_save_click(null, null);
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
        void show_msg(bool self, message m, int count)
        {
            if (InvokeRequired)
            {
                Invoke(new MethodInvoker(delegate { show_msg(self, m, count); }));
                return;
            }
            RichTextBox rtb;
            if (mhash.ContainsKey(m))
            {
                rtb = (RichTextBox)mhash[m];
            }
            else
            {
                rtb = new RichTextBox();
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
            }
            rtb.Text = m.ToString();
        }
        void del_msg(message m)
        {
            if (InvokeRequired)
            {
                Invoke(new MethodInvoker(delegate { del_msg(m); }));
                return;
            }
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
