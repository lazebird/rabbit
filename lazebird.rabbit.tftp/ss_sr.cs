using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class ss_sr : ss // server read session
    {
        public ss_sr(Func<int, string, int> log, UdpClient uc, IPEndPoint r, Hashtable opts) : base(log, uc, r, opts)
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
                    pkt_rrq pkt = new pkt_rrq();
                    if (!pkt.parse(buf)) return false;
                    update_param(pkt.timeout * 1000 / Math.Max(idic["maxretry"], 1), pkt.blksize);
                    if (!File.Exists(sdic["cwd"] + pkt.filename))
                    {
                        pktbuf = new pkt_err(Errcodes.FileNotFound, pkt.filename).pack();
                        uc.Send(pktbuf, pktbuf.Length, r);
                        filename = pkt.filename; // set filename for log
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
                        pktbuf = new pkt_data(++blkno, data).pack();
                    }
                }
                else // ack
                {
                    pkt_ack pkt = new pkt_ack();
                    if (!pkt.parse(buf)) return false;
                    if (pkt.blkno != (blkno & 0xffff)) return true;    // ignore expired ack?
                    data = q.consume();  //while ((data = q.consume()) == null) ; // may be infinite wait for a new msg here?
                    pktbuf = new pkt_data(++blkno, data).pack();
                }
            }
            else
            {
                pkt_ack pkt = new pkt_ack();
                if (!pkt.parse(buf)) return false;
                if (pkt.blkno != (blkno & 0xffff))
                    return true;    // ignore expired ack?
                if (blkno == maxblkno && blkno > 0)  // over
                    return false;
                if ((data = q.consume()) == null)
                    data = new byte[0]; // srv send last empty block; client write last block
                pktbuf = new pkt_data(++blkno, data).pack();
            }
            curretry = 0;   // reset retry cnt
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;
        }
    }
}
