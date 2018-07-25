using lazebird.rabbit.fs;
using lazebird.rabbit.queue;
using System;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using static lazebird.rabbit.tftp.tftppkt;

namespace lazebird.rabbit.tftp
{
    public class rtftpc
    {
        Func<int, string, int> log;
        rfs rfs;
        int timeout = 200;
        int maxretry = 10;
        int blksize = 1024;
        string localFile = null;

        public rtftpc(Func<int, string, int> log)
        {
            this.log = log;
            this.rfs = new rfs(slog);
        }
        public rtftpc(Func<int, string, int> log, int timeout, int maxretry, int blksize) : this(log)
        {
            this.timeout = timeout;
            this.maxretry = maxretry;
            this.blksize = blksize;
        }
        void slog(string msg) { log(0, msg); }
        bool pkt_proc(tftppkt pkt, tftpsession s, string oper)
        {
            try
            {
                if (s.blkno == 0)   // proc the first pkt recvd
                {
                    rqueue q = new rqueue(2000, 1000); // 2000 * pkt.blksize, max memory used 2M, 1000ms timeout
                    FileStream fs = null;
                    Thread t = null;
                    if (pkt.op == Opcodes.OAck) // oack for wrq/rrq
                    {
                        if (oper == "get")
                        {
                            fs = new FileStream(localFile, FileMode.Create, FileAccess.Write, FileShare.Read);
                            s.set_file(Path.GetFileName(fs.Name), fs.Length, q, pkt.timeout, pkt.blksize);
                            t = new Thread(() => rfs.writestream(fs, q, localFile));
                        }
                        else
                        {
                            pkt.op = Opcodes.Ack; // change code to ack to reply a data pkt; too special
                            fs = new FileStream(localFile, FileMode.Open, FileAccess.Read);
                            s.set_file(Path.GetFileName(fs.Name), fs.Length, q, pkt.timeout, pkt.blksize);
                            t = new Thread(() => rfs.readstream(fs, q, s.blksize));
                        }
                    }
                    else if (pkt.op == Opcodes.Data) // ack for rrq
                    {
                        s.blkno = 1; // may be a oack with no blkno, may be a data with blkno 1
                        fs = new FileStream(localFile, FileMode.Create, FileAccess.Write, FileShare.Read);
                        s.set_file(Path.GetFileName(fs.Name), fs.Length, q, 0, 0);
                        t = new Thread(() => rfs.writestream(fs, q, localFile));
                    }
                    else if (pkt.op == Opcodes.Ack) // ack for wrq
                    {
                        fs = new FileStream(localFile, FileMode.Open, FileAccess.Read);
                        s.set_file(Path.GetFileName(fs.Name), fs.Length, q, 0, 0);
                        t = new Thread(() => rfs.readstream(fs, q, s.blksize));
                    }
                    if (t != null)
                    {
                        t.IsBackground = true;
                        t.Start();
                    }
                }
                return s.reply(pkt);
            }
            catch (Exception e)
            {
                slog("!E: proc " + e.ToString());
                return false;
            }

        }
        public void get(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            this.localFile = localFile;
            tftpsession s;
            tftppkt pkt;
            byte[] buf;
            try
            {
                s = new tftpsession(new UdpClient(), new IPEndPoint(IPAddress.Parse(srvip), srvport), maxretry, timeout);
                buf = new tftppkt(Opcodes.Read, remoteFile, tftpmode.ToString(), timeout * maxretry / 1000, blksize).pack();
                s.uc.Send(buf, buf.Length, s.r);
                s.pkt = buf;    // retry rrq
                pkt = new tftppkt();
            }
            catch (Exception e)
            {
                slog("!E: get " + e.ToString());
                return;
            }

            while (true)
            {
                try
                {
                    buf = s.uc.Receive(ref s.r);   // change remote port automatically
                    if (!pkt.parse(buf) || !pkt_proc(pkt, s, "get")) break;
                }
                catch (Exception)
                {
                    if (!s.retry()) break;
                }
                s.progress_display(log);
            }
            if (pkt.errmsg != null) slog("!E: " + pkt.errmsg); // parse error
            s.session_display(log);
            s.destroy();
        }
        public void put(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            this.localFile = localFile;
            tftpsession s;
            tftppkt pkt;
            byte[] buf;
            try
            {
                s = new tftpsession(new UdpClient(), new IPEndPoint(IPAddress.Parse(srvip), srvport), maxretry, timeout);
                buf = new tftppkt(Opcodes.Write, remoteFile, tftpmode.ToString(), timeout * maxretry / 1000, blksize).pack();
                s.uc.Send(buf, buf.Length, s.r);
                s.pkt = buf;    // retry wrq
                pkt = new tftppkt();
            }
            catch (Exception e)
            {
                slog("!E: put " + e.ToString());
                return;
            }
            while (true)
            {
                try
                {
                    buf = s.uc.Receive(ref s.r);   // change remote port automatically
                    if (!pkt.parse(buf) || !pkt_proc(pkt, s, "put")) break;
                }
                catch (Exception)
                {
                    if (!s.retry()) break;
                }
                s.progress_display(log);
            }
            if (pkt.errmsg != null) slog("!E: " + pkt.errmsg); // parse error
            s.session_display(log);
            s.destroy();
        }
    }
}
