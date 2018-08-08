using System;
using System.Collections;
using System.Net;
using System.Net.Sockets;
using System.Threading;

namespace lazebird.rabbit.chat
{
    class rchat
    {
        Action<string> log;
        Thread rchatd;
        UdpClient uc;
        string username;
        Hashtable sshash;
        Hashtable ephash;
        public rchat(Action<string> log)
        {
            this.log = log;
            username = Environment.UserName + "@" + Environment.MachineName;
            sshash = new Hashtable();
            ephash = new Hashtable();
        }
        public void set_name(string name)
        {
            username = name;
        }
        void add_user(IPEndPoint ep, string user)
        {
            log("!: " + ep.ToString() + user);
            if (ephash.ContainsKey(ep)) ephash.Remove(ep);
            ephash.Add(ep, user);
            // add button here?
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
                    switch (p.type)
                    {
                        case "query":
                            pkt n = new query_response_pkt(username);
                            byte[] buf = n.pack();
                            uc.SendAsync(buf, buf.Length, r);
                            break;
                        case "query_response":
                            add_user(r, p.user);
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
        }
        public void new_chat(IPEndPoint r, string ruser)
        {
            if (sshash.ContainsKey(r)) return;
            ss s = new ss(log, username, r, ruser);
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
        }
    }
}
