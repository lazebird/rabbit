using System;
using System.Collections;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    public class rtftpd : IDisposable
    {
        Func<int, string, int> log;
        Thread tftpd;
        UdpClient uc;
        string cwd = "";    // used for wrq, set the first non-empty dir added as root dir
        Hashtable opts;
        object obj;
        public rtftpd(Func<int, string, int> log)
        {
            this.log = log;
            obj = new object();
        }
        int ilog(int line, string msg)
        {
            int ret;
            lock (obj)
                ret = log(line, msg);
            return ret;
        }
        void slog(string msg) { log(-1, msg); }
        public void set_cwd(string path)
        {
            cwd = path + "/";
        }
        void session_task(byte[] buf, IPEndPoint r)
        {
            try
            {
                ss s;
                Socket socket = new Socket(AddressFamily.InterNetworkV6, SocketType.Dgram, ProtocolType.Udp);
                socket.SetSocketOption(SocketOptionLevel.IPv6, SocketOptionName.IPv6Only, false);
                UdpClient s_uc = new UdpClient();
                s_uc.Client = socket;
                if ((Opcodes)buf[1] == Opcodes.Read)
                    s = new srss(log, cwd, s_uc, r, opts);
                else if ((Opcodes)buf[1] == Opcodes.Write)
                    s = new swss(log, cwd, s_uc, r, opts);
                else if ((Opcodes)buf[1] == Opcodes.ReadDir)
                    s = new srds(log, cwd, s_uc, r, opts);
                else
                    return;
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
            }
            catch (Exception e)
            {
                slog("!E: " + e.ToString());
            }
        }
        void daemon_task(int port)
        {
            try
            {
                Socket socket = new Socket(AddressFamily.InterNetworkV6, SocketType.Dgram, ProtocolType.Udp);
                socket.SetSocketOption(SocketOptionLevel.IPv6, SocketOptionName.IPv6Only, false);
                socket.Bind(new IPEndPoint(IPAddress.IPv6Any, port));
                uc = new UdpClient();
                uc.Client = socket;
                //uc = new UdpClient(port);
                IPEndPoint r = new IPEndPoint(IPAddress.Any, port);
                byte[] rcvBuffer;
                while (true)
                {
                    rcvBuffer = uc.Receive(ref r);
                    Thread t = new Thread(() => session_task(rcvBuffer, r));
                    t.IsBackground = true;
                    t.Start();
                }
            }
            catch (Exception e)
            {
                slog("!E: daemon " + e.ToString());
            }
        }
        public void start(int port, Hashtable opts)
        {
            this.opts = opts;
            if (tftpd == null)
            {
                tftpd = new Thread(() => daemon_task(port));
                tftpd.IsBackground = true;
                tftpd.Start();
            }
        }
        public void stop()
        {
            try
            {
                Dispose();
            }
            catch (Exception e)
            {
                slog("!E: " + e.ToString());
            }
        }
        protected virtual void Dispose(bool disposing)
        {
            if (uc != null) uc.Close();
            if (tftpd != null) tftpd.Abort();
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
