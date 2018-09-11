using System;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    public class rtftpc
    {
        Func<int, string, int> log;
        int timeout = 200;
        int maxretry = 10;
        int blksize = 1024;

        public rtftpc(Func<int, string, int> log)
        {
            this.log = log;
        }
        public rtftpc(Func<int, string, int> log, int timeout, int maxretry, int blksize) : this(log)
        {
            this.timeout = timeout;
            this.maxretry = maxretry;
            this.blksize = blksize;
        }
        void slog(string msg) { log(-1, msg); }
        void oper(Opcodes oper, string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            try
            {
                ss s;
                byte[] buf;
                if (oper == Opcodes.Read) // get
                {
                    s = new crss(localFile, new UdpClient(), new IPEndPoint(IPAddress.Parse(srvip), srvport), maxretry, timeout);
                    buf = new rrq_pkt(remoteFile, tftpmode.ToString(), timeout * maxretry / 1000, blksize).pack();
                }
                else if (oper == Opcodes.Write) // put
                {
                    s = new cwss(localFile, new UdpClient(), new IPEndPoint(IPAddress.Parse(srvip), srvport), maxretry, timeout);
                    buf = new wrq_pkt(remoteFile, tftpmode.ToString(), timeout * maxretry / 1000, blksize).pack();
                }
                else if (oper == Opcodes.ReadDir)
                {
                    s = new crds(remoteFile, new UdpClient(), new IPEndPoint(IPAddress.Parse(srvip), srvport), maxretry, timeout);
                    buf = new rdq_pkt(remoteFile).pack();
                }
                else
                {
                    return;
                }
                s.uc.Send(buf, buf.Length, s.r);
                s.pktbuf = buf;
                while (true)
                {
                    try
                    {
                        buf = s.uc.Receive(ref s.r);   // change remote port automatically
                        if (!s.pkt_proc(buf)) break;
                    }
                    catch (Exception)
                    {
                        if (!s.retry()) break;
                    }
                    s.progress_display(log);
                }
                s.session_display(log);
                s.destroy(log);
            }
            catch (Exception e)
            {
                slog("!E: " + e.ToString());
            }
        }
        bool is_dirpath(string s)
        {
            return s == "" || s[s.Length - 1] == '/';
        }
        public void get(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            if (is_dirpath(remoteFile))
                oper(Opcodes.ReadDir, srvip, srvport, remoteFile, localFile, tftpmode);
            else
                oper(Opcodes.Read, srvip, srvport, remoteFile, localFile, tftpmode);
        }
        public void put(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            oper(Opcodes.Write, srvip, srvport, remoteFile, localFile, tftpmode);
        }
    }
}
