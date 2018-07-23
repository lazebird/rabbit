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
            if (curtm - s.logtm > 100 || blkno == s.blkmax)
            {
                s.logtm = curtm;
                s.logidx = ilog(s.logidx, "I: " + s.r.ToString() + " " + s.filename + ": " + s.blkmax + "/" + blkno + ((s.curretry == 0) ? "" : (" retry " + s.curretry)));
            }
        }
        void session_display(tftpsession s)
        {
            int deltatm = (Environment.TickCount - s.starttm) / 1000;
            if (deltatm == 0) deltatm = 1;
            string msg = "I: " + s.r.ToString() + " " + s.filename + " ";
            msg += (s.blkmax == s.blkno) ? "Succ; " : "Fail; ";
            msg += s.blkmax + "/" + s.blkno + "/" + deltatm.ToString("###,###.0") + "s ";
            msg += "@" + (s.blkno / deltatm).ToString("###,###.0") + " pps/" + (s.len / deltatm).ToString("###,###.0") + " Bps; ";   // +1 to avoid divide 0
            msg += s.totalretry + " retries";
            ilog(s.logidx, msg);
        }
        void retry_data_block(tftpsession s, int blkno)
        {
            s.totalretry++;
            byte[] pkt = new tftppkt(Opcodes.Data, blkno, s.blkdata).pack();
            s.uc.Send(pkt, pkt.Length, s.r);
            progress_display(s, blkno);
            //slog("I: retransmit block " + blkno + " current " + s.blkno + " pkt blkno " + (blkno & 0xffff));
        }
        bool send_data_block(tftpsession s, int blkno)
        {
            byte[] data = s.q.consume();
            if (data == null) data = new byte[0];
            byte[] pkt = new tftppkt(Opcodes.Data, blkno, data).pack();
            progress_display(s, blkno);
            s.blkno = blkno;
            s.blkdata = data;
            s.curretry = 0;
            return s.uc.Send(pkt, pkt.Length, s.r) == pkt.Length;
        }
        bool send_err(tftpsession s, Errcodes err, string msg)
        {
            byte[] buf = new tftppkt(Opcodes.Error, err, msg).pack();
            s.uc.Send(buf, buf.Length, s.r);
            return false;
        }
        bool pkt_proc(tftppkt pkt, tftpsession s)
        {
            if (pkt.op == Opcodes.Read || pkt.op == Opcodes.Write)
            {
                if (!fhash.ContainsKey(pkt.filename))
                    return send_err(s, Errcodes.FileNotFound, pkt.filename);
                rqueue q = new rqueue(2000, 1000); // 2000 * 512, max memory used 1M, 1000ms timeout
                FileStream fs = new FileStream(((rfile)fhash[pkt.filename]).path, FileMode.Open, FileAccess.Read);
                s.set_file(pkt.filename, fs.Length, q);
                Thread t = new Thread(() => rfs.readstream(fs, q, 512));    // 10000000, max block size 10M
                t.IsBackground = true;
                t.Start();
                return send_data_block(s, 1);
            }
            if (pkt.op == Opcodes.Ack)
            {
                if (pkt.blkno == (s.blkno & 0xffff))    // max 2 bytes in pkt
                    if (s.blkno == s.blkmax)  // over
                        return false;
                    else
                        return send_data_block(s, s.blkno + 1);
                return true; // expired pkt?
            }
            return send_err(s, Errcodes.UnknownTrans, s.r.ToString() + " " + pkt.op.ToString());
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
            while (s.curretry > 0 || (pkt.parse(rcvBuffer) && pkt_proc(pkt, s)))
            {
                try
                {
                    rcvBuffer = s.uc.Receive(ref r);
                    s.curretry = 0;
                }
                catch (Exception e)
                {
                    if (++s.curretry > s.maxretry) break;
                    retry_data_block(s, s.blkno);
                }
            }
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
