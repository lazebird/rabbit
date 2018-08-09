using System;
using System.Collections;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.chat
{
    public class rchat : IDisposable
    {
        Action<string> log;
        Action<IPEndPoint, string> adduser;
        Thread rchatd;
        UdpClient uc;
        string username;
        Hashtable chathash;
        public rchat(Action<string> log, Action<IPEndPoint, string> adduser)
        {
            this.log = log;
            this.adduser = adduser;
            username = Environment.UserName + "@" + Environment.MachineName;
            chathash = new Hashtable();
        }
        public void set_name(string name)
        {
            if (string.IsNullOrEmpty(name)) return;
            username = name;
            log("I: set name " + username);
        }
        void show_notification(IPEndPoint ep, string user, string msg)
        {
            MessageBox.Show(msg, user + " " + ep.ToString());
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
                            if (chathash.ContainsKey(r)) break;
                            rchatform s = new rchatform(log, username, p, r);
                            Thread t = new Thread(() => Application.Run(s));
                            t.IsBackground = true;
                            t.Start();
                            chathash.Add(r, t);
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
            pkt p = new ntf_pkt(username, "test");
            IPEndPoint r = new IPEndPoint(IPAddress.Broadcast, port);
            byte[] buf = p.pack();
            uc.SendAsync(buf, buf.Length, r);
        }
        public void new_chat(IPEndPoint r, string ruser)
        {
            if (chathash.ContainsKey(r)) return;
            rchatform s = new rchatform(log, username, ruser, r);
            Thread t = new Thread(() => Application.Run(s));
            t.IsBackground = true;
            t.Start();
            chathash.Add(r, t);
        }
        public void start(int port)
        {
            rchatd = new Thread(() => server_task(port));
            rchatd.IsBackground = true;
            rchatd.Start();
        }
        public void stop()
        {
            Dispose();
        }
        protected virtual void Dispose(bool disposing)
        {
            if (rchatd != null) rchatd.Abort();
            rchatd = null;
            if (uc != null) uc.Dispose();
            uc = null;
            if (!disposing) return;
            foreach (Thread t in chathash.Values)
            {
                t.Abort();
            }
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
