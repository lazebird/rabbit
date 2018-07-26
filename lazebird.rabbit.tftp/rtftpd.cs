using lazebird.rabbit.fs;
using System;
using System.Collections;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    public class rtftpd
    {
        Func<int, string, int> log;
        Thread tftpd;
        UdpClient uc;
        string cwd = "";    // used for wrq, set the first non-empty dir added as root dir
        int timeout;
        int maxretry;
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
            slog("I: CWD " + cwd);
        }
        void session_task(byte[] buf, IPEndPoint r)
        {
            try
            {

                ss s;
                if ((Opcodes)buf[1] == Opcodes.Read)
                    s = new srss(cwd, new UdpClient(), r, maxretry, timeout);
                else if ((Opcodes)buf[1] == Opcodes.Write)
                    s = new swss(cwd, new UdpClient(), r, maxretry, timeout);
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
                        s.progress_display(ilog);
                    }
                s.session_display(ilog);
                s.destroy(ilog);
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
                uc = new UdpClient(port);
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
        public void start(int port, int timeout, int maxretry)
        {
            this.timeout = timeout;
            this.maxretry = maxretry;
            if (tftpd == null)
            {
                tftpd = new Thread(() => daemon_task(port));
                tftpd.IsBackground = true;
                tftpd.Start();
            }
        }
        public void start(int port)
        {
            start(port, 200, 30);
        }
        public void stop()
        {
            try
            {
                tftpd.Abort();
                tftpd = null;
                uc.Close();
                uc.Dispose();
            }
            catch (Exception e)
            {
                slog("!E: " + e.Message);
            }
        }
    }
}
