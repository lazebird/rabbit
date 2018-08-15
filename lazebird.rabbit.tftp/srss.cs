using System;
using System.IO;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class srss : ss // server read session
    {
        public srss(string cwd, UdpClient uc, IPEndPoint r, int maxretry, int timeout) : base(cwd, uc, r, maxretry, timeout)
        {
        }

        override public bool pkt_proc(byte[] buf)
        {
            byte[] data;
            if (blkno == 0)
            {
                Opcodes op = (Opcodes)buf[1];
                if (op == Opcodes.Read)
                {
                    rrq_pkt pkt = new rrq_pkt();
                    if (!pkt.parse(buf)) return false;
                    set_param(pkt.timeout * 1000 / Math.Max(maxretry, 1), pkt.blksize);
                    if (!File.Exists(cwd + pkt.filename))
                    {
                        pktbuf = new err_pkt(Errcodes.FileNotFound, pkt.filename).pack();
                        uc.Send(pktbuf, pktbuf.Length, r);
                        return false;
                    }
                    read_file(pkt.filename);
                    if (pkt.has_opt())
                    {
                        pkt.op = Opcodes.OAck;
                        pktbuf = pkt.pack(); // wait for ack blkno=0
                    }
                    else
                    {
                        data = q.consume();  //while ((data = q.consume()) == null) ; // may be infinite wait for a new msg here?
                        pktbuf = new data_pkt(++blkno, data).pack();
                    }
                }
                else // ack
                {
                    ack_pkt pkt = new ack_pkt();
                    if (!pkt.parse(buf)) return false;
                    if (pkt.blkno != (blkno & 0xffff)) return true;    // ignore expired ack?
                    data = q.consume();  //while ((data = q.consume()) == null) ; // may be infinite wait for a new msg here?
                    pktbuf = new data_pkt(++blkno, data).pack();
                }
            }
            else
            {
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
            curretry = 0;   // reset retry cnt
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;
        }
    }
}
