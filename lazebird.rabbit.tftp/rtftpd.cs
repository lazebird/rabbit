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
        int timeout = 200;
        int maxretry = 30;
        int qsize = 2000;
        bool override_flag = false;
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
                if ((Opcodes)buf[1] == Opcodes.Read)
                    s = new srss(cwd, new UdpClient(), r, maxretry, timeout, qsize);
                else if ((Opcodes)buf[1] == Opcodes.Write)
                    s = new swss(cwd, new UdpClient(), r, maxretry, timeout, override_flag);
                else if ((Opcodes)buf[1] == Opcodes.ReadDir)
                    s = new srds(cwd, new UdpClient(), r, maxretry, timeout);
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
        void parse_args(Hashtable opts)
        {
            if (opts.ContainsKey("timeout")) int.TryParse((string)opts["timeout"], out timeout);
            if (opts.ContainsKey("retry")) int.TryParse((string)opts["retry"], out maxretry);
            if (opts.ContainsKey("qsize")) int.TryParse((string)opts["qsize"], out qsize);
            if (opts.ContainsKey("override")) bool.TryParse((string)opts["override"], out override_flag);
        }
        public void start(int port, Hashtable opts)
        {
            parse_args(opts);
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
