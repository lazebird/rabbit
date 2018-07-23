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
        public rtftpd(Func<int, string, int> log) : this(log, 3, 5000)
        {
        }
        public rtftpd(Func<int, string, int> log, int maxretry, int timeout)
        {
            this.log = log;
            this.maxretry = maxretry;
            this.timeout = timeout;
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
                if (s.curretry == 0)
                    s.logidx = ilog(s.logidx, "I: " + s.ep.ToString() + " " + s.filename + ": " + s.blkmax + "/" + blkno);
                else
                    s.logidx = ilog(s.logidx, "I: " + s.ep.ToString() + " " + s.filename + ": " + s.blkmax + "/" + blkno + " retry " + s.curretry);
            }
        }
        void end_session(tftpsession s)
        {
            int curtm = Environment.TickCount;
            slog("I: " + s.filename + " "
                + s.blkno + " p/" + ((curtm - s.starttm) / 1000).ToString("###,###.00") + " s; @"
                + (s.blkno / ((curtm - s.starttm) / 1000 + 1)).ToString("###,###.00") + " pps" + ", total retry " + s.totalretry); // +1 to avoid divide 0
            s.stop_timer();
            chash.Remove(s.ep);
        }
        void send_data_block(int blkno, tftpsession s)
        {
            byte[] data;
            if (s.blkno == blkno)   // retry
            {
                s.totalretry++;
                data = s.blkdata;
                if (++s.curretry > s.maxretry)   // fail
                {
                    end_session(s);
                    return;
                }
            }
            else
            {
                data = s.q.consume();
                if (data == null) data = new byte[0];
                s.blkno = blkno;
                s.blkdata = data;
                s.curretry = 0;
                s.stop_timer();
                s.t = new Timer(p => send_data_block(blkno, s), null, s.timeout, s.timeout);
            }
            byte[] pkt = new tftppkt(Opcodes.Data, blkno, data).pack();
            uc.Send(pkt, pkt.Length, s.ep);
            progress_display(s, blkno);
        }
        void send_err(IPEndPoint r, int err, string msg)
        {
            byte[] buf = new tftppkt(Opcodes.Error, err, msg).pack();
            uc.SendAsync(buf, buf.Length, r);
        }
        void recv_task(int port)
        {
            IPEndPoint r = new IPEndPoint(IPAddress.Any, port);
            byte[] rcvBuffer;
            try
            {
                rcvBuffer = uc.Receive(ref r);
            }
            catch (Exception) { return; }
            tftppkt pkt = new tftppkt();
            if (!pkt.parse(rcvBuffer))
                return;
            if (pkt.op == Opcodes.Read || pkt.op == Opcodes.Write)
            {
                if (!fhash.ContainsKey(pkt.filename))
                {
                    send_err(r, 1, "unknown " + pkt.filename);
                    return;
                }
                if (chash.ContainsKey(r)) // reset
                {
                    chash.Remove(r);
                }
                rqueue q = new rqueue(2000, 3000); // 2000 * 512, max memory used 1M, 3000ms timeout
                FileStream fs = new FileStream(((rfile)fhash[pkt.filename]).path, FileMode.Open, FileAccess.Read);
                tftpsession s = new tftpsession(r, ((int)fs.Length + 512) / 512, maxretry, timeout, pkt.filename, q); // if size % 512 = 0, an empty data pkt sent at last
                chash.Add(r, s);
                Thread t = new Thread(() => rfs.readstream(fs, q, 512));    // 10000000, max block size 10M
                t.IsBackground = true;
                t.Start();
                send_data_block(s.blkno + 1, s);
            }
            else if (pkt.op == Opcodes.Ack && chash.ContainsKey(r))
            {
                tftpsession s = (tftpsession)chash[r];
                if (pkt.blkno == (s.blkno & 0xffff))    // max 2 bytes in pkt
                {
                    if (s.blkno == s.blkmax)  // over
                        end_session(s);
                    else
                        send_data_block(s.blkno + 1, s);
                }
            }
            else  // error
            {
                send_err(r, 2, "code " + pkt.op.ToString());
            }
        }
        void server_task(int port)
        {
            try
            {
                uc = new UdpClient(port);
            }
            catch (Exception e)
            {
                slog("!E: " + e.Message);
                return;
            }
            while (true) recv_task(port);
        }
        public void start(int port)
        {
            if (tftpd == null)
            {
                tftpd = new Thread(() => server_task(port));
                tftpd.IsBackground = true;
                tftpd.Start();
            }
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
