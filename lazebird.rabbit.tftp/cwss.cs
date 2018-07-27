using System;
using System.IO;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class cwss : ss // client write session
    {
        public cwss(string localfile, UdpClient uc, IPEndPoint r, int maxretry, int timeout) : base(Path.GetDirectoryName(localfile) + "/", uc, r, maxretry, timeout)
        {
            this.filename = Path.GetFileName(localfile);
        }
        override public bool pkt_proc(byte[] buf)
        {
            byte[] data;
            if (blkno == 0 && (Opcodes)buf[1] == Opcodes.OAck) // oack
            {
                oack_pkt pkt = new oack_pkt();
                if (!pkt.parse(buf)) return false;
                set_param(pkt.timeout * 1000 / Math.Max(maxretry, 1), pkt.blksize);
                read_file(filename);
                data = q.consume();  //while ((data = q.consume()) == null) ; // may be infinite wait for a new msg here?
                pktbuf = new data_pkt(++blkno, data).pack();
            }
            else
            {
                if (blkno == 0) read_file(filename);
                ack_pkt pkt = new ack_pkt();
                if (!pkt.parse(buf)) return false;
                if (pkt.blkno != (blkno & 0xffff))
                    return true;    // ignore expired ack?
                if (blkno == maxblkno && blkno > 0)  // over
                    return false;
                if ((data = q.consume()) == null)
                    data = new byte[0]; // srv send last empty block; client write last block
                pktbuf = new data_pkt(++blkno, data).pack();
            }
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;
        }
    }
}
