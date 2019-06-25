using lazebird.rabbit.fs;
using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class ss : IDisposable // session
    {
        public UdpClient uc;
        public IPEndPoint r;
        public byte[] pktbuf = null;
        protected Dictionary<string, string> sdic;
        protected Dictionary<string, int> idic;
        protected Dictionary<string, bool> bdic;
        protected Func<int, string, int> log;
        protected string filename = null;
        protected long filesize = 0;
        protected rqueue q = null;
        Thread t = null;
        protected int blkno = 0, maxblkno = 0;
        protected int logidx = -1, logtm = 0;
        protected int curretry = 0, totalretry = 0;
        protected int starttm = Environment.TickCount;
        public ss(Func<int, string, int> log, UdpClient uc, IPEndPoint r, Hashtable opts)
        {
            this.log = log;
            this.uc = uc;
            this.r = r;
            init_args(opts);
            uc.Client.ReceiveTimeout = idic["timeout"];
        }
        void slog(string msg) { log?.Invoke(-1, msg); }
        void init_args(Hashtable opts)
        {
            sdic = new Dictionary<string, string>();
            idic = new Dictionary<string, int>();
            bdic = new Dictionary<string, bool>();
            sdic.Add("cwd", Environment.CurrentDirectory);
            idic.Add("blksize", 512);
            idic.Add("timeout", 200); // ms
            idic.Add("retry", 10);
            idic.Add("qsize", 2000);
            idic.Add("qtout", 1000);    // ms
            bdic.Add("override", false);
            bdic.Add("fslog", false);
            foreach (string key in opts.Keys)
            {
                if (sdic.ContainsKey(key)) sdic[key] = (string)opts[key];
                else if (idic.ContainsKey(key)) idic[key] = int.Parse((string)opts[key]);
                else if (bdic.ContainsKey(key)) bdic[key] = bool.Parse((string)opts[key]);
            }
            if (sdic["cwd"].Length > 0 && sdic["cwd"][sdic["cwd"].Length - 1] != '/') sdic["cwd"] += "/"; // fix dir ending
        }
        public void update_param(int timeout, int blksize)
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
        public static ss get_srv_session(Func<int, string, int> log, byte[] buf, IPEndPoint r, Hashtable opts)
        {
            Socket socket = new Socket(AddressFamily.InterNetworkV6, SocketType.Dgram, ProtocolType.Udp);
            socket.SetSocketOption(SocketOptionLevel.IPv6, SocketOptionName.IPv6Only, false);
            UdpClient s_uc = new UdpClient();
            s_uc.Client = socket;
            if ((Opcodes)buf[1] == Opcodes.Read)
                return new ss_sr(log, s_uc, r, opts);
            else if ((Opcodes)buf[1] == Opcodes.Write)
                return new ss_sw(log, s_uc, r, opts);
            else if ((Opcodes)buf[1] == Opcodes.ReadDir)
                return new ss_srd(log, s_uc, r, opts);
            else
              throw new ArgumentException();
        }
        public static ss get_clnt_session(string localFile, string remoteFile, Opcodes oper, Modes tftpmode, Func<int, string, int> log, ref byte[] buf, IPEndPoint r, Hashtable opts)
        {
            int timeout = 200;
            int maxretry = 10;
            int blksize = 1024;
            if (opts.ContainsKey("timeout")) int.TryParse((string)opts["timeout"], out timeout);
            if (opts.ContainsKey("retry")) int.TryParse((string)opts["retry"], out maxretry);
            if (opts.ContainsKey("blksize")) int.TryParse((string)opts["qsize"], out blksize);
            if (oper == Opcodes.Read) // get
            {
                buf = new pkt_rrq(remoteFile, tftpmode.ToString(), timeout * maxretry / 1000, blksize).pack();
                return new ss_cr(localFile, log, new UdpClient(r.AddressFamily), r, opts);
            }
            else if (oper == Opcodes.Write) // put
            {
                buf = new pkt_wrq(remoteFile, tftpmode.ToString(), timeout * maxretry / 1000, blksize).pack();
                return new ss_cw(localFile, log, new UdpClient(r.AddressFamily), r, opts);
            }
            else if (oper == Opcodes.ReadDir)
            {
                buf = new pkt_rdq(remoteFile).pack();
                return new ss_crd(remoteFile, log, new UdpClient(r.AddressFamily), r, opts);
            }
            else
                throw new ArgumentException();
        }
    }
}
