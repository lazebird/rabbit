using System;
using System.Collections;
using System.Drawing;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.chat
{
    class ss : IDisposable
    {
        Action<string> log;
        public UdpClient uc;
        public IPEndPoint r;
        string user;
        string ruser;
        ArrayList msglst;
        Thread t_ui;
        Thread t_com;
        Hashtable pkthash;
        Form chatform;
        Panel logpannel;
        RichTextBox chatbox;
        public ss(Action<string> log, string user, IPEndPoint r, string ruser)
        {
            this.log = log;
            this.user = user;
            this.r = r;
            this.ruser = ruser;
            uc = new UdpClient(r);
            msglst = new ArrayList();
            pkthash = new Hashtable();
        }
        [STAThread]
        void ui_task()
        {
            chatform = new Form();
            chatform.Text = ruser + " " + r.ToString();
            chatform.Width = 800;
            chatform.Height = 600;
            chatform.Load += chat_resize;
            chatform.Resize += chat_resize;
            logpannel = new Panel();
            logpannel.DoubleClick += msg_save_click;
            chatform.Controls.Add(logpannel);
            chatbox = new RichTextBox();
            chatbox.KeyUp += msg_send_click;
            logpannel.BackColor = Color.FromArgb(64, 64, 64);
            chatbox.Font = new Font(chatbox.Font.FontFamily, 12, chatbox.Font.Style);
            chatbox.BorderStyle = BorderStyle.None;
            chatform.Controls.Add(chatbox);
            Application.EnableVisualStyles();
            Application.Run(chatform);
        }
        void msg_save_click(object sender, EventArgs e)
        {
            log("I: saving log");
            SaveFileDialog fd = new SaveFileDialog();
            fd.Filter = "文本文件 (*.txt)|*.txt|All files (*.*)|*.*";
            fd.CreatePrompt = true;
            fd.OverwritePrompt = true;
            if (fd.ShowDialog() == DialogResult.OK)
            {
                log("I: saving log to " + fd.FileName);
                log2txt(fd.FileName);
            }
        }
        void chat_resize(object sender, EventArgs e)
        {
            logpannel.Location = new Point(5, 0);
            logpannel.Width = this.chatform.Width - 10;
            logpannel.Height = (int)(this.chatform.Height * 0.6);
            chatbox.Location = new Point(5, logpannel.Height);
            chatbox.Width = this.chatform.Width - 10;
            chatbox.Height = (int)(this.chatform.Height * 0.4);
        }
        void msg_send_click(object sender, KeyEventArgs e)
        {
            RichTextBox tb = (RichTextBox)sender;
            if (e.KeyData == (Keys.Enter))
            {
                if (!string.IsNullOrEmpty(tb.Text.Trim())) write_msg(tb.Text.Trim());
                tb.Clear();
            }
            else if (e.KeyData == Keys.Escape)
            {
                chatform.Close();
            }
        }
        void com_task()
        {
            try
            {
                byte[] rcvBuffer;
                pkt p = new pkt();
                while (true)
                {
                    rcvBuffer = uc.Receive(ref r);
                    p.parse(rcvBuffer);
                    pkt_proc(p);
                }
            }
            catch (Exception e)
            {
                log("!E: session " + e.ToString());
            }
        }
        void start_com_task()
        {
            if (t_com != null) return;
            t_com = new Thread(com_task);
            t_com.IsBackground = true;
            t_com.Start();
        }
        void add_msg(bool self, string msg)
        {
            if (chatform.InvokeRequired)
            {
                chatform.Invoke(new MethodInvoker(delegate { add_msg(self, msg); }));
                return;
            }
            message m = new message(self ? user : ruser, msg);
            TextBox tb = new TextBox();
            tb.Location = new Point(5, 25 * msglst.Count);
            tb.Width = logpannel.Width - 20;
            tb.Font = new Font(tb.Font.FontFamily, 12, tb.Font.Style);
            tb.BorderStyle = BorderStyle.None;
            tb.BackColor = logpannel.BackColor;
            tb.ForeColor = Color.White;
            if (self) tb.TextAlign = HorizontalAlignment.Right;
            logpannel.Controls.Add(tb);
            tb.Text = msg;
            msglst.Add(m);
        }
        void read_msg(string pktid, string msg)
        {
            pkt pkt = new message_response_pkt(user, pktid);
            byte[] buf = pkt.pack();
            uc.SendAsync(buf, buf.Length, r);
            add_msg(false, msg);
            start_com_task();
            log("I: read msg " + msg);
        }
        public bool pkt_proc(pkt p)
        {
            switch (p.type)
            {
                case "message":
                    read_msg(p.id, p.content);
                    break;
                case "message_response":
                    break;
                default:
                    break;
            }
            return true;
        }
        int pktid = 0;
        void write_msg(string msg)
        {
            pkt pkt = new message_pkt(user, (++pktid).ToString(), msg);
            byte[] buf = pkt.pack();
            uc.SendAsync(buf, buf.Length, r);
            add_msg(true, msg);
            start_com_task();
            log("I: write msg " + msg);
        }
        public void start()
        {
            t_ui = new Thread(ui_task);
            t_ui.IsBackground = true;
            t_ui.Start();
        }
        void log2txt(string logpath)
        {
            FileStream fs = new FileStream(logpath, FileMode.Create, FileAccess.Write, FileShare.Read);
            byte[] buf = Encoding.Default.GetBytes(ruser + " & " + user);
            fs.Write(buf, 0, buf.Length);
            foreach (message m in msglst)
            {
                buf = Encoding.Default.GetBytes(m.ToString() + "\r\n");
                fs.Write(buf, 0, buf.Length);
            }
            fs.Close();
            log("I: chat log saved to " + logpath);
        }
        public void destroy()
        {
            if (t_com != null) t_com.Abort();
            t_com = null;
            if (t_ui != null) t_ui.Abort();
            t_ui = null;
        }
        protected virtual void Dispose(bool disposing)
        {
            if (t_com != null) t_com.Abort();
            t_com = null;
            if (t_ui != null) t_ui.Abort();
            t_ui = null;
            if (uc != null) uc.Dispose();
            uc = null;
            if (!disposing) return;
            if (msglst != null) msglst.Clear();
            if (chatform != null) chatform.Dispose();
            chatform = null;
            if (logpannel != null) logpannel.Dispose();
            logpannel = null;
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
