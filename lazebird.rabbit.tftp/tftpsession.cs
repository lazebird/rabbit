using lazebird.rabbit.queue;
using System;
using System.Net;
using System.Net.Sockets;
using System.Text;
using static lazebird.rabbit.tftp.tftppkt;

namespace lazebird.rabbit.tftp
{
    class tftpsession
    {
        public UdpClient uc;
        public IPEndPoint r;
        public int blksize;
        public int blkno;
        public int blkmax;
        public bool blkzero;
        public byte[] pkt;
        public int maxretry;
        public int curretry;
        public int totalretry;
        public int timeout; // ms
        public string filename;
        public long len;
        public rqueue q;
        public int starttm;
        public int logidx;
        public int logtm;

        public tftpsession(UdpClient uc, IPEndPoint r, int maxretry, int timeout)
        {
            uc.Client.ReceiveTimeout = timeout;
            this.uc = uc;
            this.r = r;
            this.maxretry = maxretry;
            this.timeout = timeout;
            this.blksize = 512; // default
            this.blkno = 0;
            this.blkzero = false;
            this.pkt = null;
            this.curretry = 0;
            this.totalretry = 0;
            this.starttm = Environment.TickCount;
            this.logidx = 0;
            this.logtm = 0;
        }

        public tftpsession(UdpClient uc, IPEndPoint r, int blkmax, int maxretry, int timeout, string filename, rqueue q) : this(uc, r, maxretry, timeout)
        {
            this.blkmax = blkmax;
            this.filename = filename;
            this.q = q;
        }
        public void set_file(string filename, long len, rqueue q, int timeout, int blksize)
        {
            this.filename = filename;
            this.len = len;
            this.q = q;
            if (timeout > 0)
            {
                this.timeout = timeout * 1000 / Math.Max(this.maxretry, 1);
                uc.Client.ReceiveTimeout = timeout;
            }
            if (blksize > 0)
            {
                this.blksize = blksize;
            }
            this.blkmax = (int)(len + this.blksize) / this.blksize;   // if len % blksize = 0, an empty data pkt sent at last
            this.blkzero = (len % this.blksize == 0);
        }
        public bool reply(tftppkt p)
        {
            if ((p.op == Opcodes.Read || p.op == tftppkt.Opcodes.Write) && (p.timeout != 0 || p.blksize != 0))
            {
                p.op = Opcodes.OAck;
                pkt = p.pack();
            }
            else if (p.op == Opcodes.Ack)
            {
                if (p.blkno != (blkno & 0xffff))
                    return true;    // ignore expired ack?
                if (blkno == blkmax)  // over
                    return false;
                byte[] data = null;
                if (blkno != blkmax - 1 || !blkzero)
                    data = q.consume(); // may be infinite wait for a new msg here?
                if (data == null) data = new byte[0];
                pkt = new tftppkt(Opcodes.Data, ++blkno, data).pack();
            }
            else
            {
                return error(Errcodes.UnknownTrans, r.ToString() + " " + p.op.ToString());
            }
            curretry = 0;
            return uc.Send(pkt, pkt.Length, r) == pkt.Length;
        }
        public bool retry()
        {
            if (++curretry > maxretry) return false;
            totalretry++;
            return uc.Send(pkt, pkt.Length, r) == pkt.Length;

        }
        public bool error(Errcodes err, string msg)
        {
            pkt = new tftppkt(Opcodes.Error, err, msg).pack();
            uc.Send(pkt, pkt.Length, r);
            return false;
        }
        public void destroy()
        {
            uc.Close();
            uc.Dispose();
        }
    }
}
