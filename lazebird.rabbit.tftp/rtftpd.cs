using lazebird.rabbit.fs;
using lazebird.rabbit.queue;
using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using static lazebird.rabbit.tftp.tftplib;

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
        void send_data_block(int blkno, tftpsession s)
        {
            int curtm = Environment.TickCount;
            if (curtm - s.logtm > 100 || blkno == s.blkmax)
            {
                s.logtm = curtm;
                s.logidx = ilog(s.blkno == blkno ? 0 : s.logidx, "I: " + s.ep.ToString() + " " + s.filename + ": " + s.blkmax + "/" + blkno);
            }
            byte[] data;
            if (s.blkno == blkno)   // retry
            {
                data = s.blkdata;
                if (++s.curretry == s.maxretry)   // fail
                {
                    s.stop_timer();
                    chash.Remove(s.ep);
                    return;
                }
            }
            else
            {
                while (true)
                {
                    data = s.q.consume();
                    if (data != null) break;
                }
                s.blkno = blkno;
                s.blkdata = data;
                s.curretry = 0;
                s.stop_timer();
                s.t = new Timer(p => send_data_block(blkno, s), null, s.timeout, s.timeout);
            }
            byte[] pkt = CreateDataPacket(blkno, data);
            uc.Send(pkt, pkt.Length, s.ep);
            if (blkno == s.blkmax)  // over
            {
                slog("I: " + s.filename + " "
                    + s.blkmax + " p/" + ((curtm - s.starttm) / 1000).ToString("###,###.00") + " s; @"
                    + (s.blkmax / ((curtm - s.starttm) / 1000 + 1)).ToString("###,###.00") + " pps"); // +1 to avoid divide 0
                s.stop_timer();
                chash.Remove(s.ep);
                return;
            }
        }
        void send_err(IPEndPoint r, int err, string msg)
        {
            byte[] buf = CreateErrPacket(err, msg);
            uc.SendAsync(buf, buf.Length, r);
        }
        void pkt_handler(int port)
        {
            IPEndPoint r = new IPEndPoint(IPAddress.Any, port);
            byte[] rcvBuffer = new byte[516];
            rcvBuffer = uc.Receive(ref r);
            Opcodes op = (Opcodes)rcvBuffer[1];
            if (op == Opcodes.Read || op == Opcodes.Write)
            {
                int pos = 2;
                while (rcvBuffer[pos] != 0) pos++;
                string vpath = "/" + Encoding.ASCII.GetString(rcvBuffer, 2, pos - 2);
                slog("I: " + op.ToString() + " " + vpath);
                if (!fhash.ContainsKey(vpath))
                {
                    send_err(r, 1, "unknown " + vpath);
                    return;
                }
                if (chash.ContainsKey(r)) // reset
                {
                    chash.Remove(r);
                }
                rqueue q = new rqueue(2000); // 2000 * 512, max memory used 1M
                FileStream fs = new FileStream(((rfile)fhash[vpath]).path, FileMode.Open, FileAccess.Read);
                tftpsession s = new tftpsession(r, ((int)fs.Length + 511) / 512, 3, 1000, vpath, q);
                chash.Add(r, s);
                new Thread(() => rfs.readfile(fs, q, 512)).Start();    // 10000000, max block size 10M
                new Thread(() => send_data_block(s.blkno + 1, s)).Start();
            }
            else if (op == Opcodes.Ack && chash.ContainsKey(r))
            {
                tftpsession s = (tftpsession)chash[r];
                int ackno = rcvBuffer[2] << 8 | rcvBuffer[3];
                if (ackno == s.blkno)
                    new Thread(() => send_data_block(s.blkno + 1, s)).Start();
            }
            else  // error
            {
                send_err(r, 2, "code " + op.ToString());
            }
        }
        void server_task(int port)
        {
            try
            {
                uc = new UdpClient(port);
                while (true) pkt_handler(port);
            }
            catch (Exception e)
            {
                slog("!E: " + e.Message);
            }
        }
        public void start(int port)
        {
            if (tftpd == null)
            {
                tftpd = new Thread(() => server_task(port));
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
