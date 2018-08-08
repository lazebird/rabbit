using System;
using System.Collections;
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
            t = new Thread(session_task);
            t.IsBackground = true;
            t.Start();
        }
        void ui_task()
        {
            f = new Form();
            Application.Run(f);
        }
        void session_task()
        {
            try
            {
                t_ui = new Thread(ui_task);
                t_ui.IsBackground = true;
                t_ui.Start();
                uc = new UdpClient();
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
                log("!E: daemon " + e.ToString());
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
    }
}
