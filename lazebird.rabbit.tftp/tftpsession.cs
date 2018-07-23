using lazebird.rabbit.queue;
using System;
using System.Net;
using System.Net.Sockets;
using System.Threading;

namespace lazebird.rabbit.tftp
{
    class tftpsession
    {
        public UdpClient uc;
        public IPEndPoint r;
        public int blkno;
        public int blkmax;
        public byte[] blkdata;
        public int maxretry;
        public int curretry;
        public int totalretry;
        public int timeout; // ms
        public string filename;
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
            this.blkno = 0;
            this.blkdata = null;
            this.curretry = 0;
            this.totalretry = 0;
            this.starttm = Environment.TickCount;
            this.logidx = 0;
            this.logtm = 0;
        }

        public tftpsession(UdpClient uc, IPEndPoint r, int blkmax, int maxretry, int timeout, string filename, rqueue q):this(uc, r, maxretry, timeout)
        {
            this.blkmax = blkmax;
            this.filename = filename;
            this.q = q;
        }
        public void set_file(string filename, long len, rqueue q)
        {
            this.blkmax = (int)(len + 512) / 512;   // if len % 512 = 0, an empty data pkt sent at last
            this.filename = filename;
            this.q = q;
        }
        public void destroy()
        {
            uc.Close();
            uc.Dispose();
        }
    }
}
