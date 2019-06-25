using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class ss_sw : ss // server write session
    {
        public ss_sw(Func<int, string, int> log, UdpClient uc, IPEndPoint r, Hashtable opts) : base(log, uc, r, opts)
        {
        }

        override public bool pkt_proc(byte[] buf)
        {
            if (blkno == 0)
            {
                pkt_wrq pkt = new pkt_wrq();
                if (!pkt.parse(buf)) return false;
                update_param(pkt.timeout * 1000 / Math.Max(idic["maxretry"], 1), pkt.blksize);
                if (File.Exists(sdic["cwd"] + pkt.filename) && !bdic["override_flag"])
                {
                    pktbuf = new pkt_err(Errcodes.FileAlreadyExists, pkt.filename).pack();
                    uc.Send(pktbuf, pktbuf.Length, r);
                    filename = pkt.filename; // set filename for log
                    return false;
                }
                write_file(pkt.filename);
                if (pkt.has_opt())
                {
                    pkt.op = Opcodes.OAck;
                    pktbuf = pkt.pack();
                    blkno++; // wait for data blkno=1
                }
                else
                {
                    pktbuf = new pkt_ack(blkno++).pack();
                }
            }
            else
            {
                pkt_data pkt = new pkt_data();
                if (!pkt.parse(buf)) return false;
                if (pkt.blkno != (blkno & 0xffff)) return true;  // ignore expired data?
                filesize += pkt.data.Length;
                if (pkt.data.Length > 0)
                    while (q.produce(pkt.data) == 0) ; // infinit produce this data
                pktbuf = new pkt_ack(blkno++).pack();
                if (pkt.data.Length < idic["blksize"]) // stop
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
