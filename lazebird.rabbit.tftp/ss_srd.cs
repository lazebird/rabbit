﻿using System;
using System.Collections;
using System.Net;
using System.Net.Sockets;
using System.Text;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class ss_srd : ss // server read session
    {
        string dirname;
        string dirinfo;
        public ss_srd(Func<int, string, int> log, UdpClient uc, IPEndPoint r, Hashtable opts) : base(log, uc, r, opts)
        {
        }
        override public bool pkt_proc(byte[] buf)
        {
            Opcodes op = (Opcodes)buf[1];
            if (op != Opcodes.ReadDir) return false;
            pkt_rdq pkt = new pkt_rdq();
            if (!pkt.parse(buf)) return false;
            dirname = pkt.dirname;
            read_dir(dirname, ref dirinfo);
            pktbuf = new pkt_data(++blkno, Encoding.Default.GetBytes(dirinfo)).pack();
            curretry = 0;   // reset retry cnt
            return uc.Send(pktbuf, pktbuf.Length, r) == pktbuf.Length;
        }
        protected override string progress_info()
        {
            return "I: " + r.ToString() + " " + dirname + " " + dirinfo + " " + totalretry + " retries";
        }
    }
}
