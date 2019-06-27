using System;
using System.Collections;
using System.Net;
using System.Net.Sockets;
using System.Threading;

namespace lazebird.rabbit.tftp
{
    public class rtftpd : IDisposable
    {
        Func<int, string, int> log; // int index, string logmsg, int ret
        Thread tftpd;
        Hashtable sshash;    // session thread table
        UdpClient uc;
        string cwd = Environment.CurrentDirectory;    // used for wrq, set the first non-empty dir added as root dir
        Hashtable opts;
        object obj;
        public rtftpd(Func<int, string, int> log)
        {
            this.log = log;
            obj = new object();
            sshash = new Hashtable();
            opts = new Hashtable();
        }
        int ilog(int line, string msg)
        {
            int ret;
            lock (obj) ret = log(line, msg);
            return ret;
        }
        void slog(string msg) { log(-1, msg); }
        public void set_cwd(string path)
        {
            cwd = path + "/";
            opts["cwd"] = cwd;
        }
        void session_handler(byte[] buf, IPEndPoint r)
        {
            ss s = ss.get_srv_session(log, buf, r, opts);
            if (s.pkt_proc(buf))
                while (true)
                {
                    try
                    {
                        buf = s.uc.Receive(ref r);
                        if (!s.pkt_proc(buf)) break;
                    }
                    catch (Exception)
                    {
                        if (!s.retry()) break;
                        //slog("I: retransmit block " + s.blkno + " pkt blkno " + (s.blkno & 0xffff));
                    }
                    s.progress_display();
                }
            s.session_display();
            s.destroy();
            if (sshash.ContainsKey(r)) sshash.Remove(r);
        }
        void session_task(byte[] buf, IPEndPoint r)
        {
            try
            {
                session_handler(buf, r);
            }
            catch (Exception e)
            {
                slog("!E: " + e.ToString());
            }
        }
        void daemon_handler(int port)
        {
            Socket socket = new Socket(AddressFamily.InterNetworkV6, SocketType.Dgram, ProtocolType.Udp);
            socket.SetSocketOption(SocketOptionLevel.IPv6, SocketOptionName.IPv6Only, false);
            socket.Bind(new IPEndPoint(IPAddress.IPv6Any, port));
            uc = new UdpClient();
            uc.Client = socket;
            IPEndPoint r = new IPEndPoint(IPAddress.Any, port);
            byte[] rcvBuffer;
            while (true)
            {
                rcvBuffer = uc.Receive(ref r);
                if (sshash.ContainsKey(r)) continue;    // avoid repeat pkts
                Thread t = new Thread(() => session_task(rcvBuffer, r));
                t.IsBackground = true;
                t.Start();
                sshash.Add(r, t);
            }
        }
        void daemon_task(int port)
        {
            try
            {
                daemon_handler(port);
            }
            catch (Exception e)
            {
                slog("!E: daemon " + e.ToString());
            }
        }
        public void start(int port, Hashtable opts)
        {
            if (!opts.ContainsKey("cwd")) opts.Add("cwd", cwd);
            this.opts = opts;
            if (tftpd != null) throw new ArgumentException();
            tftpd = new Thread(() => daemon_task(port));
            tftpd.IsBackground = true;
            tftpd.Start();
        }
        public void stop()
        {
            Dispose();
        }
        protected virtual void Dispose(bool disposing)
        {
            if (uc != null) uc.Close();
            if (tftpd != null) tftpd.Abort();
            foreach (Thread t in sshash.Values) t.Abort();
            sshash = new Hashtable();
            uc = null;
            tftpd = null;
            if (!disposing) return;
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
