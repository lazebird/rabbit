using System;
using System.Text;

namespace lazebird.rabbit.tftp
{
    class tftplib
    {
        public enum Opcodes
        {
            Read = 1,
            Write = 2,
            Data = 3,
            Ack = 4,
            Error = 5
        }
        public static byte[] CreateReqPacket(Opcodes opCode, string remoteFile, string tftpMode)
        {
            int pos = 0;
            byte[] pkt = new byte[tftpMode.Length + remoteFile.Length + 4];
            pkt[pos++] = 0;
            pkt[pos++] = (byte)opCode;
            pos += Encoding.ASCII.GetBytes(remoteFile, 0, remoteFile.Length, pkt, pos);
            pkt[pos++] = 0;
            pos += Encoding.ASCII.GetBytes(tftpMode, 0, tftpMode.Length, pkt, pos);
            pkt[pos] = 0;
            return pkt;
        }
        public static byte[] CreateDataPacket(int blkno, byte[] data)
        {
            byte[] pkt = new byte[4 + data.Length];
            pkt[0] = 0;
            pkt[1] = (byte)Opcodes.Data;
            pkt[2] = (byte)((blkno >> 8) & 0xff);
            pkt[3] = (byte)(blkno & 0xff);
            Array.Copy(data, 0, pkt, 4, data.Length);
            return pkt;
        }

        public static byte[] CreateAckPacket(int blkno)
        {
            byte[] pkt = new byte[4];
            pkt[0] = 0;
            pkt[1] = (byte)Opcodes.Ack;
            pkt[2] = (byte)((blkno >> 8) & 0xff);
            pkt[3] = (byte)(blkno & 0xff);
            return pkt;
        }
        public static byte[] CreateErrPacket(string msg)
        {
            byte[] pkt = new byte[5 + msg.Length];
            pkt[0] = 0;
            pkt[1] = (byte)Opcodes.Error;
            pkt[2] = 0;
            pkt[3] = 0;
            Encoding.ASCII.GetBytes(msg, 0, msg.Length, pkt, 4);
            pkt[4 + msg.Length] = 0;
            return pkt;
        }
    }
}
