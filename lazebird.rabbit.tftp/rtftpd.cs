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
        string rootdir = "";    // used for wrq, set the first non-empty dir added as root dir
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
                ret = log(line, msg);
            return ret;
        }
        void slog(string msg) { log(0, msg); }
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
                    FileStream fs = new FileStream(rootdir + pkt.filename, FileMode.Create, FileAccess.Write, FileShare.Read);
                    rqueue q = new rqueue(2000, 1000); // 2000 * pkt.blksize, max memory used 2M, 1000ms timeout
                    s.set_file(pkt.filename, 0, q, pkt.timeout, pkt.blksize);
                    Thread t = new Thread(() => rfs.writestream(fs, q, pkt.filename));
                    t.IsBackground = true;
                    t.Start();
                }
                catch (Exception e)
                {
                    slog("!E: proc " + e.ToString());
                    return false;
                }
            }
            return s.reply(pkt);
        }
        void session_task(byte[] rcvBuffer, IPEndPoint r)
        {
            if (chash.ContainsKey(r)) chash.Remove(r);
            tftpsession s = new tftpsession(new UdpClient(), r, maxretry, timeout);
            chash.Add(r, s);
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
                    s.progress_display(ilog);
                }
            if (pkt.errmsg != null) slog("!E: " + pkt.errmsg); // parse error
            s.session_display(ilog);
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
            try
            {
                if (path == null || path == "")
                    return;
                if (rootdir == "")
                    rootdir = path + "/";
                DirectoryInfo dir = new DirectoryInfo(path);
                foreach (FileInfo f in dir.GetFiles())
                {
                    rfs.addfile(fhash, "/", f.FullName);
                }
            }
            catch (Exception e)
            {
                slog("!E: " + e.Message);
            }
        }
        public void del_dir(string path)
        {
            if (path == null || path == "")
                return;
            if (rootdir == path || rootdir == path + "/")
                rootdir = "";
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
