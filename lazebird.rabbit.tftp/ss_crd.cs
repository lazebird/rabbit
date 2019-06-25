using System;
using System.Collections;
using System.Net;
using System.Net.Sockets;
using System.Text;

namespace lazebird.rabbit.tftp
{
    class ss_crd : ss // client read directory session
    {
        string dirname;
        string dirinfo = "";
        public ss_crd(string dirname, Func<int, string, int> log, UdpClient uc, IPEndPoint r, Hashtable opts) : base(log, uc, r, opts)
        {
            this.dirname = dirname;
        }
        override public bool pkt_proc(byte[] buf)
        {
            if (blkno == 0) blkno++;
            pkt_data pkt = new pkt_data();
            if (!pkt.parse(buf)) return false;
            if (pkt.blkno != (blkno & 0xffff))
                return true;  // ignore expired data?
            filesize += pkt.data.Length;
            if (pkt.data.Length > 0) dirinfo = Encoding.Default.GetString(pkt.data);
            pktbuf = new pkt_ack(blkno++).pack();
            uc.Send(pktbuf, pktbuf.Length, r);
            return false;
        }
        protected override string progress_info()
        {
            return "I: " + r.ToString() + " " + dirname + " " + dirinfo + " " + totalretry + " retries";
        }
    }
}
