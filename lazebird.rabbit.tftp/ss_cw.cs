using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    class ss_cw : ss // client write session
    {
        string remoteFile;
        Modes tftpmode;
        public ss_cw(string localfile, string remoteFile, Modes tftpmode, Func<int, string, int> log, UdpClient uc, IPEndPoint r, Hashtable opts) : base(log, uc, r, opts)
        {
            this.filename = Path.GetFileName(localfile);
            sdic["cwd"] = Path.GetDirectoryName(localfile);
            if (sdic["cwd"].Length > 0 && sdic["cwd"][sdic["cwd"].Length - 1] != '/') sdic["cwd"] += "/"; // fix dir ending
            this.remoteFile = remoteFile;
            this.tftpmode = tftpmode;
        }
        override public bool pkt_proc(byte[] buf)
        {
            byte[] data;
            if (buf == null)
            {
                pktbuf = new pkt_wrq(remoteFile, tftpmode.ToString(), idic["timeout"] * idic["maxretry"] / 1000, idic["blksize"]).pack();
            }
            else if (blkno == 0 && (Opcodes)buf[1] == Opcodes.OAck) // oack
            {
                pkt_oack pkt = new pkt_oack();
                if (!pkt.parse(buf)) return false;
                update_param(pkt.timeout * 1000 / Math.Max(idic["maxretry"], 1), pkt.blksize);
                read_file(filename);
                data = q.consume();  //while ((data = q.consume()) == null) ; // may be infinite wait for a new msg here?
                pktbuf = new pkt_data(++blkno, data).pack();
            }
            else
            {
                if (blkno == 0) read_file(filename);
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
