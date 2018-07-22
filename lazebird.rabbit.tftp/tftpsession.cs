using lazebird.rabbit.queue;
using System;
using System.Net;
using System.Threading;

namespace lazebird.rabbit.tftp
{
    class tftpsession
    {
        public IPEndPoint ep;
        public int blkno;
        public int blkmax;
        public byte[] blkdata;
        public int maxretry;
        public int curretry;
        public int timeout; // ms
        public string filename;
        public rqueue q;
        public Timer t;
        public int starttm;
        public int logidx;
        public int logtm;
        public tftpsession(IPEndPoint ep, int blkmax, int maxretry, int timeout, string filename, rqueue q)
        {
            this.ep = ep;
            this.blkmax = blkmax;
            this.maxretry = maxretry;
            this.timeout = timeout;
            this.filename = filename;
            this.q = q;
            this.blkno = 0;
            this.blkdata = null;
            this.curretry = 0;
            this.starttm = Environment.TickCount;
            this.logidx = 0;
            this.logtm = 0;
        }
        public void stop_timer()
        {
            if (t != null)
            {
                t.Change(Timeout.Infinite, Timeout.Infinite);
                t.Dispose();
                t = null;
            }
        }
    }
}
