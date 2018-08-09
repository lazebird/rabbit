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
            pa_chat.DoubleClick += msg_save_click;
            rtb_chat.KeyUp += msg_key_press;
        }
        SaveFileDialog fd;
        void msg_save_click(object sender, EventArgs e)
        {
            fd = new SaveFileDialog();
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
        void show_msg(bool self, message m, int count)
        {
            if (InvokeRequired)
            {
                Invoke(new MethodInvoker(delegate { show_msg(self, m, count); }));
                return;
            }
            TextBox tb;
            if (mhash.ContainsKey(m))
            {
                tb = (TextBox)mhash[m];
            }
            else
            {
                tb = new TextBox();
                tb.Location = new Point(5, 25 * count);
                tb.Width = pa_chat.Width - 20;
                tb.Font = new Font(tb.Font.FontFamily, 12, tb.Font.Style);
                tb.BorderStyle = BorderStyle.None;
                tb.BackColor = pa_chat.BackColor;
                tb.ForeColor = Color.White;
                if (self) tb.TextAlign = HorizontalAlignment.Right;
                pa_chat.Controls.Add(tb);
                mhash.Add(m, tb);
            }
            tb.Text = m.toshortstring();
        }
        void del_msg(message m)
        {
            if (InvokeRequired)
            {
                Invoke(new MethodInvoker(delegate { del_msg(m); }));
                return;
            }
            TextBox tb;
            if (mhash.ContainsKey(m))
            {
                tb = (TextBox)mhash[m];
                pa_chat.Controls.Remove(tb);
                mhash.Remove(m);
            }
        }
    }
}
