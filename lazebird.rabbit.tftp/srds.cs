using System;
using System.Collections;
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
        public srds(Func<int, string, int> log, string cwd, UdpClient uc, IPEndPoint r, Hashtable opts) : base(log, cwd, uc, r, opts)
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
        protected override string progress_info()
        {
            return "I: " + r.ToString() + " " + dirname + " " + dirinfo + " " + totalretry + " retries";
        }
    }
}
