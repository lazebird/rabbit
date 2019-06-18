using lazebird.rabbit.fs;
using System;
using System.Collections.Generic;
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
        Dictionary<string, string> sdic;
        Dictionary<string, int> idic;
        Dictionary<string, bool> bdic;
        protected Func<int, string, int> log;
        protected string filename = null;
        protected long filesize = 0;
        protected rqueue q = null;
        Thread t = null;
        protected int blkno = 0, maxblkno = 0;
        protected int logidx = -1, logtm = 0;
        protected int curretry = 0, totalretry = 0;
        protected int starttm = Environment.TickCount;
        public ss(Func<int, string, int> log, string cwd, UdpClient uc, IPEndPoint r, Dictionary<string, string> sdic, Dictionary<string, int> idic, Dictionary<string, bool> bdic)
        {
            this.log = log;
            this.sdic = sdic;
            this.idic = idic;
            this.bdic = bdic;
            parse_args();
            this.uc = uc;
            this.r = r;
            uc.Client.ReceiveTimeout = idic["timeout"];
        }
        void slog(string msg) { log?.Invoke(-1, msg); }
        void parse_args()
        {
            if (!sdic.ContainsKey("cwd")) sdic.Add("cwd", "");
            if (sdic["cwd"].Length > 0 && sdic["cwd"][sdic["cwd"].Length - 1] != '/') sdic["cwd"] += "/"; // fix dir ending
            if (!idic.ContainsKey("blksize")) idic.Add("blksize", 512);
            if (!idic.ContainsKey("timeout")) idic.Add("timeout", 200); // ms
            if (!idic.ContainsKey("retry")) idic.Add("retry", 10);
            if (!idic.ContainsKey("qsize")) idic.Add("qsize", 2000);
            if (!idic.ContainsKey("qtout")) idic.Add("qtout", 1000);    // ms
            if (!bdic.ContainsKey("override")) bdic.Add("override", false);
            if (!bdic.ContainsKey("fslog")) bdic.Add("fslog", false);
        }
        public void set_param(int timeout, int blksize)
        {
            idic["timeout"] = timeout;
            idic["blksize"] = blksize;
            uc.Client.ReceiveTimeout = timeout;
        }
        virtual public void rfs_log(string msg) { }
        public void read_file(string filename)
        {
            FileStream fs = new FileStream(sdic["cwd"] + filename, FileMode.Open, FileAccess.Read);
            this.filename = filename;
            this.filesize = fs.Length;
            this.q = new rqueue(idic["qsize"], idic["qtout"]);
            maxblkno = (int)(this.filesize + idic["blksize"]) / idic["blksize"];   // if len % blksize = 0, an empty data pkt sent at last
            if (bdic["fslog"]) t = new Thread(() => rfs.readstream_log(fs, this.q, idic["blksize"], this.filename, slog));
            else t = new Thread(() => rfs.readstream(fs, this.q, idic["blksize"]));
            t.IsBackground = true;
            t.Start();
        }
        public void write_file(string filename)
        {
            FileStream fs = new FileStream(sdic["cwd"] + filename, FileMode.Create, FileAccess.Write, FileShare.Read);
            this.filename = filename;
            this.q = new rqueue(idic["qsize"], idic["qtout"]);
            if (bdic["fslog"]) t = new Thread(() => rfs.writestream_log(fs, this.q, this.filename, slog));
            else t = new Thread(() => rfs.writestream(fs, this.q, this.filename));
            t.IsBackground = true;
            t.Start();
        }
        public void read_dir(string dirname, ref string dirinfo)
        {
            if (!Directory.Exists(sdic["cwd"] + dirname)) return;
            DirectoryInfo d = new DirectoryInfo(sdic["cwd"] + dirname);
            dirinfo = "";
            foreach (FileInfo f in d.GetFiles()) dirinfo += f.Name + ";";
            this.filename = dirname;
            this.filesize = dirinfo.Length;
        }
        virtual public bool pkt_proc(byte[] buf) { return false; }
        public bool retry()
        {
            if (++curretry > (idic["retry"])) return false;
            totalretry++;
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;
        }
        public void progress_display()
        {
            int curtm = Environment.TickCount;
            if (curtm - logtm > 100 || blkno == maxblkno)
            {
                logtm = curtm;
                logidx = log(logidx, "I: " + r.ToString() + " " + filename + ": " + maxblkno + " / " + blkno + ((curretry == 0) ? "" : (" retry " + curretry)));
            }
        }
        protected virtual string progress_info()
        {
            int deltatm = Math.Max(1, (Environment.TickCount - starttm) / 1000);
            long curlen = (maxblkno == blkno) ? filesize : (idic["blksize"] * blkno);
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
        }

        public void destroy()
        {
            logidx = log(logidx, progress_info() + " (destroyed)");
            if (q != null) q.stop();
            if (t != null) t.Join();    // when write file, t may be still running when tx is over
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
