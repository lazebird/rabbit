using System;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class crss : ss // client read session
    {
        public crss(string cwd, UdpClient uc, IPEndPoint r, int maxretry, int timeout) : base(cwd, uc, r, maxretry, timeout)
        {
        }

        override public bool pkt_proc(byte[] buf)
        {
            if (blkno == 0 && (Opcodes)buf[1] == Opcodes.OAck) // oack
            {
                oack_pkt pkt = new oack_pkt();
                if (!pkt.parse(buf)) return false;
                set_param(pkt.timeout * 1000 / Math.Max(maxretry, 1), pkt.blksize);
                pktbuf = new ack_pkt(blkno++).pack();
            }
            else
            {
                data_pkt pkt = new data_pkt();
                if (!pkt.parse(buf)) return false;
                if (pkt.blkno != (blkno & 0xffff))
                    return true;  // ignore expired data?
                filesize += pkt.data.Length;
                if (pkt.data.Length > 0)
                    while (q.produce(pkt.data) == 0) ; // infinit produce this data
                pktbuf = new ack_pkt(blkno++).pack();
                if (pkt.data.Length < blksize) // stop
                {
                    maxblkno = --blkno;
                    q.stop();
                    uc.Send(pktbuf, pktbuf.Length, r);
                    return false;
                }
            }
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;
        }
    }
}
