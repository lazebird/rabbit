using System;
using System.Text;

namespace lazebird.rabbit.tftp
{
    class tftppkt
    {
        public enum Opcodes
        {
            Read = 1,
            Write = 2,
            Data = 3,
            Ack = 4,
            Error = 5
        }
        public enum Modes
        {
            netascii,
            octet,
            mail
        }
        public Opcodes op;
        public string filename;
        public string mode;
        public int blkno;
        public byte[] data;
        public int errno;
        public string errmsg;
        public tftppkt()
        {
            this.op = 0;
            this.filename = null;
            this.mode = null;
            this.blkno = 0;
            this.data = null;
            this.errno = 0;
            this.errmsg = null;
        }

        public tftppkt(Opcodes op, string filename, string mode)    // req
        {
            this.op = op;
            this.filename = filename;
            this.mode = mode;
        }

        public tftppkt(Opcodes op, int blkno, byte[] data)  // data
        {
            this.op = op;
            this.blkno = blkno;
            this.data = data;
        }

        public tftppkt(Opcodes op, int blkno)   // ack
        {
            this.op = op;
            this.blkno = blkno;
        }

        public tftppkt(Opcodes op, int errno, string errmsg)    // err
        {
            this.op = op;
            this.errno = errno;
            this.errmsg = errmsg;
        }

        public bool parse(byte[] buf)
        {
            op = (Opcodes)buf[1];
            if (op == Opcodes.Read || op == Opcodes.Write)
            {
                int filenameend = 2;
                while (buf[filenameend] != 0) filenameend++;
                filename = "/" + Encoding.ASCII.GetString(buf, 2, filenameend - 2);
                int modeend = filenameend + 1;
                while (buf[modeend] != 0) modeend++;
                mode = Encoding.ASCII.GetString(buf, filenameend + 1, modeend - filenameend - 1);
            }
            else if (op == Opcodes.Data)
            {
                blkno = buf[2] << 8 | buf[3];
                data = new byte[buf.Length - 4];
                Array.Copy(buf, 4, data, 0, buf.Length - 4);
            }
            else if (op == Opcodes.Ack)
            {
                blkno = buf[2] << 8 | buf[3];
            }
            else if (op == Opcodes.Error)
            {
                errno = buf[2] << 8 | buf[3];
                errmsg = Encoding.ASCII.GetString(buf, 4, buf.Length - 5);
            }
            else
                return false;
            return true;
        }
        public byte[] pack()
        {
            byte[] buf = null;
            if (op == Opcodes.Read || op == Opcodes.Write)
            {
                buf = new byte[mode.Length + filename.Length + 4];
                int pos = 0;
                buf[pos++] = 0;
                buf[pos++] = (byte)op;
                pos += Encoding.ASCII.GetBytes(filename, 0, filename.Length, buf, pos);
                buf[pos++] = 0;
                pos += Encoding.ASCII.GetBytes(mode, 0, mode.Length, buf, pos);
                buf[pos] = 0;
            }
            else if (op == Opcodes.Data)
            {
                buf = new byte[4 + data.Length];
                buf[0] = 0;
                buf[1] = (byte)Opcodes.Data;
                buf[2] = (byte)((blkno >> 8) & 0xff);
                buf[3] = (byte)(blkno & 0xff);
                Array.Copy(data, 0, buf, 4, data.Length);
            }
            else if (op == Opcodes.Ack)
            {
                buf = new byte[4];
                buf[0] = 0;
                buf[1] = (byte)Opcodes.Ack;
                buf[2] = (byte)((blkno >> 8) & 0xff);
                buf[3] = (byte)(blkno & 0xff);
            }
            else if (op == Opcodes.Error)
            {
                buf = new byte[5 + errmsg.Length];
                buf[0] = 0;
                buf[1] = (byte)Opcodes.Error;
                buf[2] = 0;
                buf[3] = (byte)errno;
                Encoding.ASCII.GetBytes(errmsg, 0, errmsg.Length, buf, 4);
                buf[4 + errmsg.Length] = 0;
            }
            return buf;
        }
    }
}
