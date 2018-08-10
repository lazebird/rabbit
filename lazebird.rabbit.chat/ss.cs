using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Threading;

namespace lazebird.rabbit.chat
{
    public class ss : IDisposable
    {
        Action<string> log;
        public UdpClient uc;
        public IPEndPoint r;
        string luser;
        string ruser;
        ArrayList msglst;
        Thread t_com;
        Hashtable pkthash;
        Action<bool, message, int> show_msg;
        Action<message> del_msg;
        public ss(Action<string> log, string luser, string ruser, IPEndPoint r)
        {
            this.log = log;
            this.luser = luser;
            this.ruser = ruser;
            this.r = r;
            uc = new UdpClient(r);
            msglst = new ArrayList();
            pkthash = new Hashtable();
        }
        public void init_view_api(Action<bool, message, int> show_msg, Action<message> del_msg)
        {
            this.show_msg = show_msg;
            this.del_msg = del_msg;
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
        void read_msg(string pktid, string msg)
        {
            message m = new message(ruser, msg);
            msglst.Add(m);
            show_msg(false, m, msglst.Count);
            pkt pkt = new message_response_pkt(luser, pktid);
            byte[] buf = pkt.pack();
            uc.SendAsync(buf, buf.Length, r);
            start_com_task();
        }
        public void write_msg(string msg)
        {
            message m = new message(luser, msg);
            msglst.Add(m);
            show_msg(true, m, msglst.Count);
            pkt pkt = new message_pkt(luser, (++pktid).ToString(), msg);
            byte[] buf = pkt.pack();
            uc.SendAsync(buf, buf.Length, r);
            start_com_task();
        }
        public void log2txt(string logpath)
        {
            FileStream fs = new FileStream(logpath, FileMode.Create, FileAccess.Write, FileShare.Read);
            byte[] buf = Encoding.Default.GetBytes(ruser + " & " + luser + "\r\n");
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
            Dispose();
        }
        protected virtual void Dispose(bool disposing)
        {
            if (t_com != null) t_com.Abort();
            t_com = null;
            if (uc != null) uc.Dispose();
            uc = null;
            if (!disposing) return;
            if (msglst != null) msglst.Clear();
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
