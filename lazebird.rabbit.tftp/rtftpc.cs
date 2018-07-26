using System;
using System.IO;
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
                    s = new crss(Path.GetDirectoryName(localFile), new UdpClient(), new IPEndPoint(IPAddress.Parse(srvip), srvport), maxretry, timeout);
                    s.write_file(Path.GetFileName(localFile));
                    buf = new rrq_pkt(remoteFile, tftpmode.ToString(), timeout * maxretry / 1000, blksize).pack();
                }
                else // put
                {
                    s = new cwss(Path.GetDirectoryName(localFile), new UdpClient(), new IPEndPoint(IPAddress.Parse(srvip), srvport), maxretry, timeout);
                    s.read_file(Path.GetFileName(localFile));
                    buf = new wrq_pkt(remoteFile, tftpmode.ToString(), timeout * maxretry / 1000, blksize).pack();
                }
                s.uc.Send(buf, buf.Length, s.r);
                s.pktbuf = buf;    // retry wrq
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
                slog("!E: put " + e.ToString());
            }
        }
        public void get(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            oper(Opcodes.Read, srvip, srvport, remoteFile, localFile, tftpmode);

        }
        public void put(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            oper(Opcodes.Write, srvip, srvport, remoteFile, localFile, tftpmode);
        }
    }
}
