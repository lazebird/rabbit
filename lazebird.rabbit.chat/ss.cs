using System;
using System.Collections;
using System.Drawing;
using System.Net;
using System.Net.Sockets;
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
        Form f;
        Hashtable pkthash;
        Panel p;
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
        void ui_task()
        {
            f = new Form();
            f.Text = ruser + " " + r.ToString();
            f.Width = 800;
            f.Height = 600;
            f.Load += F_Load;
            Application.Run(f);
        }

        void F_Load(object sender, EventArgs e)
        {
            FlowLayoutPanel fp = new FlowLayoutPanel();
            fp.Width = f.Width;
            fp.Height = f.Height;
            f.Controls.Add(fp);
            p = new Panel();
            p.Width = f.Width - 10;
            p.Height = (int)(f.Height * 0.6);
            p.BackColor = Color.FromArgb(64, 64, 64);
            fp.Controls.Add(p);
            TextBox tb = new TextBox();
            tb.BorderStyle = BorderStyle.None;
            tb.BackColor = Color.Gray;
            tb.ForeColor = Color.White;
            tb.Multiline = true;
            tb.WordWrap = true;
            tb.Width = f.Width - 10;
            tb.Height = (int)(f.Height * 0.4);
            tb.KeyDown += Tb_KeyDown;
            fp.Controls.Add(tb);
        }

        void Tb_KeyDown(object sender, KeyEventArgs e)
        {
            TextBox tb = (TextBox)sender;
            if (e.KeyData == (Keys.Control | Keys.Enter))
            {
                write_msg(tb.Text);
                tb.Text = "";
            }
            else if (e.KeyData == Keys.Escape)
            {
                f.Close();
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
        void add_msg(string owner, string msg)
        {
            if (f.InvokeRequired)
            {
                f.Invoke(new MethodInvoker(delegate { add_msg(owner, msg); }));
                return;
            }
            message m = new message(owner, msg);
            TextBox tb = new TextBox();
            tb.Location = new Point(5, 25 * msglst.Count);
            tb.Width = p.Width - 20;
            tb.BorderStyle = BorderStyle.None;
            tb.BackColor = p.BackColor;
            tb.ForeColor = Color.White;
            if (owner == user) tb.TextAlign = HorizontalAlignment.Right;
            p.Controls.Add(tb);
            tb.Text = msg;
            msglst.Add(m);
        }
        void read_msg(string pktid, string msg)
        {
            pkt pkt = new message_response_pkt(user, pktid);
            byte[] buf = pkt.pack();
            uc.SendAsync(buf, buf.Length, r);
            add_msg(ruser, msg);
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
            add_msg(user, msg);
            start_com_task();
            log("I: write msg " + msg);
        }
        public void start()
        {
            t_ui = new Thread(ui_task);
            t_ui.IsBackground = true;
            t_ui.Start();
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
            if (f != null) f.Dispose();
            f = null;
            if (p != null) p.Dispose();
            p = null;
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
