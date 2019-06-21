using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class crss : ss // client read session
    {
        public crss(string localfile, Func<int, string, int> log, UdpClient uc, IPEndPoint r, Hashtable opts) : base(log, uc, r, opts)
        {
            this.filename = Path.GetFileName(localfile);
        }
        override public bool pkt_proc(byte[] buf)
        {
            if (blkno == 0 && (Opcodes)buf[1] == Opcodes.OAck) // oack
            {
                oack_pkt pkt = new oack_pkt();
                if (!pkt.parse(buf)) return false;
                set_param(pkt.timeout * 1000 / Math.Max(idic["maxretry"], 1), pkt.blksize);
                write_file(filename);
                pktbuf = new ack_pkt(blkno++).pack();
            }
            else
            {
                if (blkno == 0) // data blkno start with 1
                {
                    write_file(filename);
                    blkno++;
                }
                data_pkt pkt = new data_pkt();
                if (!pkt.parse(buf)) return false;
                if (pkt.blkno != (blkno & 0xffff)) return true;  // ignore expired data?
                filesize += pkt.data.Length;
                if (pkt.data.Length > 0)
                    while (q.produce(pkt.data) == 0) ; // infinit produce this data
                pktbuf = new ack_pkt(blkno++).pack();
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
