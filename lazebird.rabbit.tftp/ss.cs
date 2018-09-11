using lazebird.rabbit.fs;
using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading;

namespace lazebird.rabbit.tftp
{
    class ss : IDisposable // session
    {
        public UdpClient uc;
        public IPEndPoint r;
        public byte[] pktbuf = null;
        protected Func<int, string, int> log;
        protected string cwd; // current work directory
        protected int maxretry = 10;
        protected int timeout = 200; // ms
        protected int blksize = 512; // default
        protected int blkno = 0;
        protected string filename = null;
        protected long filesize = 0;
        protected int maxblkno = 0;
        protected int qsize = 2000;
        protected int qtout = 1000;
        protected bool override_flag = false;
        bool fslog;
        protected rqueue q = null;
        Thread t = null;
        protected int curretry = 0;
        protected int totalretry = 0;
        protected int starttm = Environment.TickCount;
        protected int logidx = -1;
        protected int logtm = 0;
        public ss(Func<int, string, int> log, string cwd, UdpClient uc, IPEndPoint r, Hashtable opts)
        {
            this.log = log;
            parse_args(opts);
            if (cwd.Length > 0 && cwd[cwd.Length - 1] != '/') cwd += "/"; // fix dir ending
            this.cwd = cwd;
            this.uc = uc;
            this.r = r;
            uc.Client.ReceiveTimeout = this.timeout;
        }
        void slog(string msg) { log?.Invoke(-1, msg); }
        void parse_args(Hashtable opts)
        {
            if (opts.ContainsKey("timeout")) int.TryParse((string)opts["timeout"], out timeout);
            if (opts.ContainsKey("retry")) int.TryParse((string)opts["retry"], out maxretry);
            if (opts.ContainsKey("qsize")) int.TryParse((string)opts["qsize"], out qsize);
            if (opts.ContainsKey("override")) bool.TryParse((string)opts["override"], out override_flag);
            if (opts.ContainsKey("fslog")) bool.TryParse((string)opts["fslog"], out fslog);
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
            this.q = new rqueue(qsize, qtout);
            this.maxblkno = (int)(this.filesize + this.blksize) / this.blksize;   // if len % blksize = 0, an empty data pkt sent at last
            if (fslog) t = new Thread(() => rfs.readstream_log(fs, this.q, this.blksize, this.filename, slog));
            else t = new Thread(() => rfs.readstream(fs, this.q, this.blksize));
            t.IsBackground = true;
            t.Start();
        }
        public void write_file(string filename)
        {
            FileStream fs = new FileStream(cwd + filename, FileMode.Create, FileAccess.Write, FileShare.Read);
            this.filename = filename;
            this.q = new rqueue(qsize, qtout);
            if (fslog) t = new Thread(() => rfs.writestream_log(fs, this.q, this.filename, slog));
            else t = new Thread(() => rfs.writestream(fs, this.q, this.filename));
            t.IsBackground = true;
            t.Start();
        }
        public void read_dir(string dirname, ref string dirinfo)
        {
            if (!Directory.Exists(cwd + dirname)) return;
            DirectoryInfo d = new DirectoryInfo(cwd + dirname);
            dirinfo = "";
            foreach (FileInfo f in d.GetFiles()) dirinfo += f.Name + ";";
            this.filename = dirname;
            this.filesize = dirinfo.Length;
        }
        virtual public bool pkt_proc(byte[] buf) { return false; }
        public bool retry()
        {
            if (++curretry > maxretry) return false;
            totalretry++;
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;

        }
        public void progress_display()
        {
            int curtm = Environment.TickCount;
            if (curtm - logtm > 100 || blkno == maxblkno)
            {
                logtm = curtm;
                logidx = log(logidx, "I: " + r.ToString() + " " + filename + ": " + maxblkno + "/" + blkno + ((curretry == 0) ? "" : (" retry " + curretry)));
            }
        }
        protected virtual string progress_info()
        {
            int deltatm = Math.Max(1, (Environment.TickCount - starttm) / 1000);
            long curlen = (maxblkno == blkno) ? filesize : (blksize * blkno);
            string msg = "I: " + r.ToString() + " " + filename + " ";
            if (maxblkno != 0 && maxblkno == blkno) msg += filesize.ToString("###,###") + "B" + "/" + deltatm.ToString("###,###.0") + "s ";
            else msg += "Fail; " + maxblkno + "/" + blkno + "/" + deltatm.ToString("###,###.0") + "s ";
            msg += "@" + (blkno / deltatm).ToString("###,###.0") + " pps/" + (curlen / deltatm).ToString("###,###.0") + " Bps; ";
            msg += totalretry + " retries";
            return msg;
        }
        public virtual void session_display()
        {
            logidx = log(logidx, progress_info());
            //if (q != null) log(-1, "I: produce " + q.stat_produce + " consume " + q.stat_consume + " stopped " + q.is_stopped());
        }

        public void destroy()
        {
            logidx = log(logidx, progress_info() + " (destroyed)");
            //log(-1, "I: Destroy session: " + r.ToString() + " " + cwd + filename);
            if (q != null) q.stop();
            if (t != null) t.Join();
            Dispose(true);
        }
        protected virtual void Dispose(bool disposing)
        {
            if (q != null) q.Dispose();
            if (uc != null) uc.Close();
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
