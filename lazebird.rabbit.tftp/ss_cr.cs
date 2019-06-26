using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class ss_cr : ss // client read session
    {
        string remoteFile;
        Modes tftpmode;
        public ss_cr(string localfile, string remoteFile, Modes tftpmode, Func<int, string, int> log, UdpClient uc, IPEndPoint r, Hashtable opts) : base(log, uc, r, opts)
        {
            this.filename = Path.GetFileName(localfile);
            this.remoteFile = remoteFile;
            this.tftpmode = tftpmode;
        }
        override public bool pkt_proc(byte[] buf)
        {
            if (buf == null)
            {
                pktbuf = new pkt_rrq(remoteFile, tftpmode.ToString(), idic["timeout"] * idic["maxretry"] / 1000, idic["blksize"]).pack();
            }
            else if (blkno == 0 && (Opcodes)buf[1] == Opcodes.OAck) // oack
            {
                pkt_oack pkt = new pkt_oack();
                if (!pkt.parse(buf)) return false;
                update_param(pkt.timeout * 1000 / Math.Max(idic["maxretry"], 1), pkt.blksize);
                write_file(filename);
                pktbuf = new pkt_ack(blkno++).pack();
            }
            else
            {
                if (blkno == 0) // data blkno start with 1
                {
                    write_file(filename);
                    blkno++;
                }
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
