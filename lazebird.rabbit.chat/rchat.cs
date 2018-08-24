﻿using System;
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
            //log("I: set name " + username);
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
                            log("I: recv chat from " + p.user + " " + r.Address + " msg " + p.content);
                            if (chathash.ContainsKey(r.Address))
                            {
                                if (((rchatform)chathash[r.Address]).IsDisposed) chathash.Remove(r.Address);
                                else
                                {
                                    ((rchatform)chathash[r.Address]).pkt_proc(p, r); // reuse old session
                                    ((rchatform)chathash[r.Address]).Activate();
                                    break;
                                }
                            }
                            rchatform f = new rchatform(log, username, p.user, r);
                            Thread t = new Thread(() => Application.Run(f));
                            t.IsBackground = true;
                            t.Start();
                            f.pkt_proc(p, r);
                            chathash.Add(r.Address, f); // keep only one session per ip 
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
        public void new_query(string ip, int port)
        {
            try
            {
                if (uc == null) return;
                pkt p = new query_pkt();
                IPEndPoint r = new IPEndPoint(IPAddress.Parse(ip), port);
                byte[] buf = p.pack();
                uc.SendAsync(buf, buf.Length, r);
            }
            catch (Exception e)
            {
                log("!E: " + e.ToString());
            }
        }
        public void new_chat(IPEndPoint r, string ruser)
        {
            if (chathash.ContainsKey(r.Address))
            {
                if (((rchatform)chathash[r.Address]).IsDisposed) chathash.Remove(r.Address);
                else return;
            }
            log("I: new chat to " + ruser + " " + r.Address);
            rchatform f = new rchatform(log, username, ruser, r);
            Thread t = new Thread(() => Application.Run(f));
            t.IsBackground = true;
            t.Start();
            chathash.Add(r.Address, f);
        }
        public void new_notification(string ip, int port)
        {
            try
            {
                rntfform f = new rntfform(log, username, new IPEndPoint(IPAddress.Parse(ip), port));
                Thread t = new Thread(() => Application.Run(f));
                t.IsBackground = true;
                t.Start();
            }
            catch (Exception e)
            {
                log("!E: " + e.ToString());
            }
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
            if (uc != null) uc.Close();
            uc = null;
            foreach (rchatform f in chathash.Values) f.Dispose();
            chathash.Clear();
            if (!disposing) return;
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
