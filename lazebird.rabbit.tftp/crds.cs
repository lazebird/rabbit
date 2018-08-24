using System;
using System.Net;
using System.Net.Sockets;
using System.Text;

namespace lazebird.rabbit.tftp
{
    class crds : ss // client read directory session
    {
        string dirname;
        string dirinfo = "";
        public crds(string dirname, UdpClient uc, IPEndPoint r, int maxretry, int timeout) : base(Environment.CurrentDirectory, uc, r, maxretry, timeout)
        {
            this.dirname = dirname;
        }
        override public bool pkt_proc(byte[] buf)
        {
            if (blkno == 0) blkno++;
            data_pkt pkt = new data_pkt();
            if (!pkt.parse(buf)) return false;
            if (pkt.blkno != (blkno & 0xffff))
                return true;  // ignore expired data?
            filesize += pkt.data.Length;
            if (pkt.data.Length > 0) dirinfo = Encoding.Default.GetString(pkt.data);
            pktbuf = new ack_pkt(blkno++).pack();
            uc.Send(pktbuf, pktbuf.Length, r);
            return false;
        }
        public override void session_display(Func<int, string, int> log)
        {
            string msg = "I: " + r.ToString() + " " + dirname + " " + dirinfo + " ";
            msg += totalretry + " retries";
            log(logidx, msg);
        }
    }
}
