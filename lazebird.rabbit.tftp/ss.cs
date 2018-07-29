using lazebird.rabbit.fs;
using lazebird.rabbit.queue;
using System;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading;

namespace lazebird.rabbit.tftp
{
    class ss : IDisposable // session
    {
        public string cwd; // current work directory
        public UdpClient uc;
        public IPEndPoint r;
        public int maxretry = 10;
        public int timeout = 200; // ms
        public int blksize = 512; // default
        public int blkno = 0;
        public string filename = null;
        public long filesize = 0;
        public int maxblkno = 0;
        public rqueue q = null;
        Thread t = null;
        public int curretry = 0;
        public int totalretry = 0;
        public int starttm = Environment.TickCount;
        public int logidx = -1;
        public int logtm = 0;
        public byte[] pktbuf = null;
        public ss(string cwd, UdpClient uc, IPEndPoint r, int maxretry, int timeout)
        {
            this.cwd = cwd;
            this.uc = uc;
            this.r = r;
            this.maxretry = maxretry;
            this.timeout = timeout;
            uc.Client.ReceiveTimeout = this.timeout;
        }
        public void set_param(int timeout, int blksize)
        {
            if (timeout > 0) this.timeout = timeout;
            if (blksize > 0) this.blksize = blksize;
            uc.Client.ReceiveTimeout = this.timeout;
        }
        virtual public void rfs_log(string msg) { }
        public void read_file(string filename)
        {
            FileStream fs = new FileStream(cwd + filename, FileMode.Open, FileAccess.Read);
            this.filename = filename;
            this.filesize = fs.Length;
            this.q = new rqueue(2000, 1000);
            this.maxblkno = (int)(this.filesize + this.blksize) / this.blksize;   // if len % blksize = 0, an empty data pkt sent at last
            t = new Thread(() => rfs.readstream(fs, this.q, this.blksize));
            t.IsBackground = true;
            t.Start();
        }
        public void write_file(string filename)
        {
            FileStream fs = new FileStream(cwd + filename, FileMode.Create, FileAccess.Write, FileShare.Read);
            this.filename = filename;
            this.q = new rqueue(2000, 1000);
            t = new Thread(() => rfs.writestream(fs, this.q, this.filename));
            t.IsBackground = true;
            t.Start();
        }
        virtual public bool pkt_proc(byte[] buf) { return false; }
        public bool retry()
        {
            if (++curretry > maxretry) return false;
            totalretry++;
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;

        }
        public void progress_display(Func<int, string, int> log)
        {
            int curtm = Environment.TickCount;
            if (curtm - logtm > 100 || blkno == maxblkno)
            {
                logtm = curtm;
                logidx = log(logidx, "I: " + r.ToString() + " " + filename + ": " + maxblkno + "/" + blkno + ((curretry == 0) ? "" : (" retry " + curretry)));
            }
        }
        public void session_display(Func<int, string, int> log)
        {
            int deltatm = Math.Max(1, (Environment.TickCount - starttm) / 1000);
            long curlen = (maxblkno == blkno) ? filesize : (filesize * blkno / Math.Max(maxblkno, 1));
            string msg = "I: " + r.ToString() + " " + filename + " ";
            msg += (maxblkno == blkno) ? "Succ; " : "Fail; ";
            msg += maxblkno + "/" + blkno + "/" + deltatm.ToString("###,###.0") + "s ";
            msg += "@" + (blkno / deltatm).ToString("###,###.0") + " pps/" + (curlen / deltatm).ToString("###,###.0") + " Bps; ";
            msg += totalretry + " retries";
            log(logidx, msg);
            //if (q != null) log(-1, "I: produce " + q.stat_produce + " consume " + q.stat_consume + " stopped " + q.is_stopped());
        }

        public void destroy(Func<int, string, int> log)
        {
            if (q != null)
                q.stop();
            if (t != null) t.Join();
            Dispose(true);
            log(-1, "I: Destroy session: " + r.ToString());
        }
        protected virtual void Dispose(bool disposing)
        {
            if (q != null) q.Dispose();
            if (uc != null) uc.Dispose();
            q = null;
            uc = null;
            if (!disposing) return;
            r = null;
            pktbuf = null;
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
