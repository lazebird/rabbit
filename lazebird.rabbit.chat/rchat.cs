using System;
using System.Collections;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.chat
{
    public class rchat
    {
        Action<string> log;
        Action<IPEndPoint, string> adduser;
        Thread rchatd;
        UdpClient uc;
        string username;
        Hashtable sshash;
        public rchat(Action<string> log, Action<IPEndPoint, string> adduser)
        {
            this.log = log;
            this.adduser = adduser;
            username = Environment.UserName + "@" + Environment.MachineName;
            sshash = new Hashtable();
        }
        public void set_name(string name)
        {
            username = name;
        }
        void show_notification(IPEndPoint ep, string user, string msg)
        {
        }
        void server_task(int port)
        {
            try
            {
                uc = new UdpClient(port);
                IPEndPoint r = new IPEndPoint(IPAddress.Any, port);
                byte[] rcvBuffer;
                pkt p = new pkt();
                while (true)
                {
                    rcvBuffer = uc.Receive(ref r);
                    p.parse(rcvBuffer);
                    log("I: recv pkt type " + p.type);
                    switch (p.type)
                    {
                        case "query":
                            pkt n = new query_response_pkt(username);
                            byte[] buf = n.pack();
                            uc.SendAsync(buf, buf.Length, r);
                            break;
                        case "query_response":
                            adduser(r, p.user);
                            break;
                        case "notification":
                            show_notification(r, p.user, p.content);
                            break;
                        case "message":
                            if (sshash.ContainsKey(r)) break;
                            ss s = new ss(log, username, r, p.user);
                            s.pkt_proc(p);
                            sshash.Add(r, s);
                            break;
                        default:
                            break;
                    }
                }
            }
            catch (Exception e)
            {
                log("!E: daemon " + e.ToString());
            }
        }
        public void send_query(int port)
        {
            if (uc == null) return;
            pkt p = new query_pkt();
            IPEndPoint r = new IPEndPoint(IPAddress.Broadcast, port);
            byte[] buf = p.pack();
            uc.SendAsync(buf, buf.Length, r);
            log("I: query " + r.ToString());
        }
        public void send_notification(int port)
        {
            if (uc == null) return;
            pkt p = new ntf_pkt();
            IPEndPoint r = new IPEndPoint(IPAddress.Broadcast, port);
            byte[] buf = p.pack();
            uc.SendAsync(buf, buf.Length, r);
        }
        public void new_chat(IPEndPoint r, string ruser)
        {
            if (sshash.ContainsKey(r)) return;
            ss s = new ss(log, username, r, ruser);
            s.start();
            sshash.Add(r, s);
        }
        public void start(int port)
        {
            rchatd = new Thread(() => server_task(port));
            rchatd.IsBackground = true;
            rchatd.Start();
        }
        public void stop()
        {
            if (rchatd != null) rchatd.Abort();
            rchatd = null;
            if (uc != null) uc.Dispose();
            uc = null;
            foreach (ss s in sshash.Values)
            {
                s.stop();
            }
        }
    }
}
