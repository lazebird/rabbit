using lazebird.rabbit.fs;
using lazebird.rabbit.queue;
using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using static lazebird.rabbit.tftp.tftppkt;

namespace lazebird.rabbit.tftp
{
    public class rtftpd
    {
        Func<int, string, int> log;
        Thread tftpd;
        UdpClient uc;
        Hashtable chash;
        Hashtable fhash;
        rfs rfs;
        int timeout;
        int maxretry;
        object obj;
        public rtftpd(Func<int, string, int> log)
        {
            this.log = log;
            chash = new Hashtable();
            fhash = new Hashtable();
            rfs = new rfs(slog);
            obj = new object();
        }
        int ilog(int line, string msg)
        {
            int ret;
            lock (obj)
            {
                ret = log(line, msg);
            }
            return ret;
        }
        void slog(string msg)
        {
            log(0, msg);
        }
        void progress_display(tftpsession s, int blkno)
        {
            int curtm = Environment.TickCount;
            if (curtm - s.logtm > 100 || blkno == s.maxblkno)
            {
                s.logtm = curtm;
                s.logidx = ilog(s.logidx, "I: " + s.r.ToString() + " " + s.filename + ": " + s.maxblkno + "/" + blkno + ((s.curretry == 0) ? "" : (" retry " + s.curretry)));
            }
        }
        void session_display(tftpsession s)
        {
            int deltatm = Math.Max(1, (Environment.TickCount - s.starttm) / 1000);
            long curlen = (s.maxblkno == s.blkno) ? s.filesize : (s.filesize * s.blkno / Math.Max(s.maxblkno, 1));
            string msg = "I: " + s.r.ToString() + " " + s.filename + " ";
            msg += (s.maxblkno == s.blkno) ? "Succ; " : "Fail; ";
            msg += s.maxblkno + "/" + s.blkno + "/" + deltatm.ToString("###,###.0") + "s ";
            msg += "@" + (s.blkno / deltatm).ToString("###,###.0") + " pps/" + (curlen / deltatm).ToString("###,###.0") + " Bps; ";   // +1 to avoid divide 0
            msg += s.totalretry + " retries";
            ilog(s.logidx, msg);
        }
        bool pkt_proc(tftppkt pkt, tftpsession s)
        {
            if (pkt.op == Opcodes.Read)
            {
                if (!fhash.ContainsKey(pkt.filepath))
                    return s.error(Errcodes.FileNotFound, pkt.filename);
                rqueue q = new rqueue(2000, 1000); // 2000 * pkt.blksize, max memory used 2M, 1000ms timeout
                FileStream fs = new FileStream(((rfile)fhash[pkt.filepath]).path, FileMode.Open, FileAccess.Read);
                s.set_file(pkt.filename, fs.Length, q, pkt.timeout, pkt.blksize);
                Thread t = new Thread(() => rfs.readstream(fs, q, s.blksize));
                t.IsBackground = true;
                t.Start();
            }
            else if (pkt.op == Opcodes.Write)
            {
                try
                {
                    FileStream fs = new FileStream(pkt.filename, FileMode.Create, FileAccess.Write);
                    rqueue q = new rqueue(2000, 1000); // 2000 * pkt.blksize, max memory used 2M, 1000ms timeout
                    s.set_file(pkt.filename, 0, q, pkt.timeout, pkt.blksize);
                    Thread t = new Thread(() => rfs.writestream(fs, q, pkt.filename));
                    t.IsBackground = true;
                    t.Start();
                }
                catch (Exception)
                {
                    return false;
                }
            }
            return s.reply(pkt);
        }
        void session_task(byte[] rcvBuffer, IPEndPoint r)
        {
            tftpsession s;
            if (!chash.ContainsKey(r))
            {
                s = new tftpsession(new UdpClient(), r, maxretry, timeout);
                chash.Add(r, s);
            }
            else
                s = (tftpsession)chash[r];
            tftppkt pkt = new tftppkt();
            if (pkt.parse(rcvBuffer) && pkt_proc(pkt, s))
                while (true)
                {
                    try
                    {
                        rcvBuffer = s.uc.Receive(ref r);
                        if (!pkt.parse(rcvBuffer) || !pkt_proc(pkt, s)) break;
                    }
                    catch (Exception)
                    {
                        if (!s.retry()) break;
                        //slog("I: retransmit block " + blkno + " current " + s.blkno + " pkt blkno " + (blkno & 0xffff));
                    }
                    progress_display(s, s.blkno);
                }
            if (pkt.errmsg != null) slog("!E: " + pkt.errmsg); // parse error
            session_display(s);
            s.destroy();
            chash.Remove(r);
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
                fhash.Clear();
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
        public void add_dir(string path)
        {
            if (path == null || path == "")
            {
                return;
            }
            DirectoryInfo dir = new DirectoryInfo(path);
            foreach (FileInfo f in dir.GetFiles())
            {
                rfs.addfile(fhash, "/", f.FullName);
            }
        }
        public void del_dir(string path)
        {
            if (path == null || path == "")
            {
                return;
            }
            DirectoryInfo dir = new DirectoryInfo(path);
            foreach (FileInfo f in dir.GetFiles())
            {
                rfs.delfile(fhash, "/", f.FullName);
            }
        }
        public void add_file(string path)
        {
            rfs.addfile(fhash, "/", path);
        }
        public void del_file(string path)
        {
            rfs.delfile(fhash, "/", path);
        }
    }
}
