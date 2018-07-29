using System;
using System.IO;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class crss : ss // client read session
    {
        public crss(string localfile, UdpClient uc, IPEndPoint r, int maxretry, int timeout) : base(Path.GetDirectoryName(localfile) + "/", uc, r, maxretry, timeout)
        {
            this.filename = Path.GetFileName(localfile);
        }
        override public bool pkt_proc(byte[] buf)
        {
            if (blkno == 0 && (Opcodes)buf[1] == Opcodes.OAck) // oack
            {
                oack_pkt pkt = new oack_pkt();
                if (!pkt.parse(buf)) return false;
                set_param(pkt.timeout * 1000 / Math.Max(maxretry, 1), pkt.blksize);
                write_file(filename);
                pktbuf = new ack_pkt(blkno++).pack();
            }
            else
            {
                if (blkno == 0) write_file(filename);
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
            curretry = 0;   // reset retry cnt
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;
        }
    }
}
