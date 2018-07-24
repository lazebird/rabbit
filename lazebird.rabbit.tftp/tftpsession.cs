using lazebird.rabbit.queue;
using System;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.tftppkt;

namespace lazebird.rabbit.tftp
{
    class tftpsession
    {
        public UdpClient uc;
        public IPEndPoint r;
        public int maxblkno;
        public int maxretry;
        public int timeout; // ms
        public string filename;
        public long filesize;
        public rqueue q;
        public int blksize = 512; // default
        public int blkno = 0;
        public byte[] pkt = null;
        public int curretry = 0;
        public int totalretry = 0;
        public int starttm = Environment.TickCount;
        public int logidx = 0;
        public int logtm = 0;

        public tftpsession(UdpClient uc, IPEndPoint r, int maxretry, int timeout)
        {
            this.uc = uc;
            this.r = r;
            this.maxretry = maxretry;
            this.timeout = timeout;
            uc.Client.ReceiveTimeout = timeout;
        }
        public void set_file(string filename, long len, rqueue q, int timeout, int blksize)
        {
            this.filename = filename;
            this.filesize = len;
            this.q = q;
            if (timeout > 0)
            {
                this.timeout = timeout * 1000 / Math.Max(this.maxretry, 1);
                uc.Client.ReceiveTimeout = this.timeout;
            }
            if (blksize > 0)
            {
                this.blksize = blksize;
            }
            this.maxblkno = (int)(len + this.blksize) / this.blksize;   // if len % blksize = 0, an empty data pkt sent at last
        }
        public bool reply(tftppkt p)
        {
            byte[] data;
            if (p.op == Opcodes.Read)
            {
                if (p.timeout != 0 || p.blksize != 0)
                {
                    p.op = Opcodes.OAck;
                    pkt = p.pack(); // wait for ack blkno=0
                }
                else
                {
                    data = q.consume();  //while ((data = q.consume()) == null) ; // may be infinite wait for a new msg here?
                    pkt = new tftppkt(Opcodes.Data, ++blkno, data).pack();
                }
            }
            else if (p.op == Opcodes.Write)
            {
                if (p.timeout != 0 || p.blksize != 0)
                {
                    p.op = Opcodes.OAck;
                    pkt = p.pack();
                    blkno++; // wait for data blkno=1
                }
                else
                {
                    pkt = new tftppkt(Opcodes.Ack, blkno++).pack();
                }
            }
            else if (p.op == Opcodes.Ack)
            {
                if (p.blkno != (blkno & 0xffff))
                    return true;    // ignore expired ack?
                if (blkno == maxblkno)  // over
                    return false;
                if (q.is_stopped())
                    data = new byte[0]; // last empty block
                else
                    data = q.consume();
                pkt = new tftppkt(Opcodes.Data, ++blkno, data).pack();
            }
            else if (p.op == Opcodes.Data)
            {
                if (p.blkno != (blkno & 0xffff))
                    return true;  // ignore expired data?
                filesize += p.data.Length;
                if (p.data.Length > 0)
                    while (q.produce(p.data) == 0) ; // infinit produce this data
                pkt = new tftppkt(Opcodes.Ack, blkno++).pack();
                if (p.data.Length < blksize) // stop
                {
                    maxblkno = --blkno;
                    q.stop();
                    uc.Send(pkt, pkt.Length, r);
                    return false;
                }
            }
            else
            {
                return error(Errcodes.UnknownTrans, p.op.ToString());
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
            if (q != null)
                q.stop();
            uc.Close();
            uc.Dispose();
        }
    }
}
