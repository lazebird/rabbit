using System;
using System.Collections;
using System.Drawing;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.chat
{
    class ss
    {
        Action<string> log;
        public UdpClient uc;
        public IPEndPoint r;
        string user;
        string ruser;
        ArrayList msglst;
        Thread t;
        Thread t_ui;
        Form f;
        Hashtable pkthash;
        public ss(Action<string> log, string user, IPEndPoint r, string ruser)
        {
            this.log = log;
            this.user = user;
            this.r = r;
            this.ruser = ruser;
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
            Panel p = new Panel();
            p.Width = f.Width - 10;
            p.Height = (int)(f.Height * 0.6);
            p.BackColor = Color.FromArgb(64, 64, 64);
            fp.Controls.Add(p);
            TextBox tb = new TextBox();
            tb.BackColor = Color.Gray;
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
                send_message(tb.Text);
                tb.Text = ""; 
            }
        }

        void session_task()
        {
            try
            {
                t_ui = new Thread(ui_task);
                t_ui.IsBackground = true;
                t_ui.Start();
                uc = new UdpClient();
                uc.Connect(r);
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
        void show_message(string msg)
        {
            message m = new message(ruser, msg);
            m.show();
            msglst.Add(m);
        }
        public bool pkt_proc(pkt p)
        {
            switch (p.type)
            {
                case "message":
                    show_message(p.content);
                    break;
                case "message_response":
                    break;
                default:
                    break;
            }
            return true;
        }
        int pktid = 0;
        void send_message(string msg)
        {
            pkt p = new message_pkt(user, (++pktid).ToString(), msg);
            byte[] buf = p.pack();
            uc.SendAsync(buf, buf.Length, r);
            message m = new message(user, msg);
            msglst.Add(m);
        }
        public void start()
        {
            t = new Thread(session_task);
            t.IsBackground = true;
            t.Start();
            log("I: session task started");
        }
        public void stop()
        {
            if (t != null) t.Abort();
            t = null;
            if (t_ui != null) t_ui.Abort();
            t_ui = null;
        }
    }
}
