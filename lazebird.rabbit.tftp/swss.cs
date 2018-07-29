﻿using System;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class swss : ss // server write session
    {
        public swss(string cwd, UdpClient uc, IPEndPoint r, int maxretry, int timeout) : base(cwd, uc, r, maxretry, timeout)
        {
        }

        override public bool pkt_proc(byte[] buf)
        {
            if (blkno == 0)
            {
                wrq_pkt pkt = new wrq_pkt();
                if (!pkt.parse(buf)) return false;
                set_param(pkt.timeout * 1000 / Math.Max(maxretry, 1), pkt.blksize);
                write_file(pkt.filename);
                if (pkt.has_opt())
                {
                    pkt.op = Opcodes.OAck;
                    pktbuf = pkt.pack();
                    blkno++; // wait for data blkno=1
                }
                else
                {
                    pktbuf = new ack_pkt(blkno++).pack();
                }
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
            curretry = 0;   // reset retry cnt
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;
        }
    }
}
