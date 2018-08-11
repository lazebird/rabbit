using System;
using System.Net;
using System.Net.Sockets;
using System.Text;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class srds : ss // server read session
    {
        string dirname;
        string dirinfo;
        public srds(string cwd, UdpClient uc, IPEndPoint r, int maxretry, int timeout) : base(cwd, uc, r, maxretry, timeout)
        {
        }
        override public bool pkt_proc(byte[] buf)
        {
            Opcodes op = (Opcodes)buf[1];
            if (op != Opcodes.ReadDir) return false;
            rdq_pkt pkt = new rdq_pkt();
            if (!pkt.parse(buf)) return false;
            dirname = pkt.dirname;
            read_dir(dirname, ref dirinfo);
            pktbuf = new data_pkt(++blkno, Encoding.Default.GetBytes(dirinfo)).pack();
            curretry = 0;   // reset retry cnt
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;
        }
        public override void session_display(Func<int, string, int> log)
        {
            string msg = "I: " + r.ToString() + " " + dirname + " " + dirinfo + " ";
            msg += totalretry + " retries";
            log(logidx, msg);
        }
    }
}
