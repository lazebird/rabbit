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
        Thread t_hear;
        Hashtable pkthash; // resp pktid -- req pkt
        Hashtable msghash; // req pkt -- msg
        Hashtable tmrhash; // req pkt -- tmr
        Hashtable retryhash; // req pkt -- cur_retry
        int maxretry = 4;
        int timeout = 500; // ms
        int rxpktid = 0;
        int txpktid = 0;
        public delegate void OnHearHandler(object sender, EventArgs e);
        public event OnHearHandler OnHear;
        public delegate void OnSayfailHandler(object sender, EventArgs e);
        public event OnSayfailHandler OnSayfail;
        public ss(Action<string> log, string luser, string ruser, IPEndPoint r)
        {
            this.log = log;
            this.luser = luser;
            this.ruser = ruser;
            this.r = r;
            uc = new UdpClient();
            msglst = new ArrayList();
            pkthash = new Hashtable();
            msghash = new Hashtable();
            tmrhash = new Hashtable();
            retryhash = new Hashtable();
        }
        void hear_task()
        {
            try
            {
                byte[] rcvBuffer;
                pkt p = new pkt();
                while (true)
                {
                    rcvBuffer = uc.Receive(ref r);
                    p.parse(rcvBuffer);
                    pkt_proc(p, r);
                }
            }
            catch (Exception e)
            {
                log("!E: session " + e.ToString());
            }
        }
        void start_hear_task()
        {
            if (t_hear != null) return;
            t_hear = new Thread(hear_task);
            t_hear.IsBackground = true;
            t_hear.Start();
        }
        public bool pkt_proc(pkt p, IPEndPoint r)
        {
            this.r = r; // update remote info dynamically
            switch (p.type)
            {
                case "message":
                    hear(p.id, p.content);
                    break;
                case "message_response":
                    int pid = int.Parse(p.id);
                    if (!pkthash.ContainsKey(pid)) break;
                    byte[] buf = (byte[])pkthash[pid];
                    pkthash.Remove(pid);
                    msghash.Remove(buf);
                    Timer retrytmr = (Timer)tmrhash[buf];
                    tmrhash.Remove(buf);
                    retrytmr.Change(Timeout.Infinite, Timeout.Infinite);
                    retrytmr.Dispose();
                    break;
                default:
                    break;
            }
            return true;
        }
        void hear(string id, string msg)
        {
            int pid = int.Parse(id);
            if (rxpktid < pid)  // not expired pkt
            {
                rxpktid = pid;
                message m = new message(ruser, msg, msglst.Count);
                msglst.Add(m);
                OnHear(m, null);
            }
            pkt pkt = new message_response_pkt(luser, id);
            byte[] buf = pkt.pack();
            uc.SendAsync(buf, buf.Length, r);
            start_hear_task();
        }
        void pkt_retry(object o)
        {
            byte[] buf = (byte[])o;
            uc.SendAsync(buf, buf.Length, r);
            retryhash[buf] = (int)retryhash[buf] + 1;
            if (((int)retryhash[buf]) > maxretry)
            {
                message m = (message)msghash[buf];
                m.set_status(true);
                OnSayfail(m, null);
                int pktid = -1;
                foreach (int id in pkthash.Keys)
                    if (pkthash[id] == buf) pktid = id;
                if (pktid > 0) pkthash.Remove(pktid);
                msghash.Remove(buf);
                Timer retrytmr = (Timer)tmrhash[buf];
                tmrhash.Remove(buf);
                retrytmr.Change(Timeout.Infinite, Timeout.Infinite);
                retrytmr.Dispose();
            }

        }
        public message say(string msg)
        {
            message m = new message(luser, msg, msglst.Count);
            msglst.Add(m);
            pkt pkt = new message_pkt(luser, (++txpktid).ToString(), msg);
            byte[] buf = pkt.pack();
            uc.SendAsync(buf, buf.Length, r);
            Timer retrytmr = new Timer(pkt_retry, buf, timeout, timeout);
            pkthash.Add(txpktid, buf);
            msghash.Add(buf, m);
            tmrhash.Add(buf, retrytmr);
            retryhash.Add(buf, 0);
            start_hear_task();
            return m;
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
            foreach (Timer tmr in tmrhash.Values) tmr.Dispose();
            tmrhash.Clear();
            if (t_hear != null) t_hear.Abort();
            t_hear = null;
            if (uc != null) uc.Close();
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
