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
            Error = 5,
            OAck = 6
        }
        public enum Modes
        {
            netascii,
            octet,
            mail
        }
        public enum Errcodes
        {
            Undefined = 0,
            FileNotFound = 1,
            AccessViolation = 2,
            DiskFull = 3,
            IllegalOper = 4,
            UnknownTrans = 5,
            FileAlreadyExists = 6,
            NoSuchUser = 7
        }
        public Opcodes op;
        public string filename;
        public string mode;
        public int blksize;
        public int timeout;
        public int blkno;
        public byte[] data;
        public Errcodes errno;
        public string errmsg;
        public tftppkt()
        {
            this.op = 0;
            this.filename = null;
            this.mode = null;
            this.blksize = 0;
            this.timeout = 0;
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

        public tftppkt(Opcodes op, Errcodes errno, string errmsg)    // err
        {
            this.op = op;
            this.errno = errno;
            this.errmsg = errmsg;
        }
        bool parse_opt(string name, string val)
        {
            if (name == "timeout")
                timeout = int.Parse(val);
            else if (name == "blksize")
                blksize = int.Parse(val);
            else
                return false;
            return true;
        }
        public bool parse(byte[] buf)
        {
            //try
            //{
            op = (Opcodes)buf[1];
            if (op == Opcodes.Read || op == Opcodes.Write)
            {
                int pos = 2;
                int end;
                end = Array.FindIndex(buf, pos, (x) => x == 0);
                filename = Encoding.Default.GetString(buf, pos, end - pos);
                pos = end + 1;
                end = Array.FindIndex(buf, pos, (x) => x == 0);
                mode = Encoding.Default.GetString(buf, pos, end - pos);
                pos = end + 1;
                while (pos < buf.Length)
                {
                    end = Array.FindIndex(buf, pos, (x) => x == 0);
                    string optname = Encoding.Default.GetString(buf, pos, end - pos);
                    pos = end + 1;
                    end = Array.FindIndex(buf, pos, (x) => x == 0);
                    string optval = Encoding.Default.GetString(buf, pos, end - pos);
                    pos = end + 1;
                    parse_opt(optname, optval);
                }
                filename = "/" + filename;
                return true;
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
                errno = (Errcodes)(buf[2] << 8 | buf[3]);
                errmsg = Encoding.Default.GetString(buf, 4, buf.Length - 5);
            }
            else
                return false;
            return true;
            //}
            //catch (Exception e)
            //{
            //    errmsg = e.ToString();
            //    return false;
            //}
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
            else if (op == Opcodes.OAck)
            {
                int len = 2;
                string timeoutstr = timeout.ToString();
                if (timeout > 0)
                {
                    len += 7 + 1 + timeoutstr.Length + 1;
                }
                string blksizestr = blksize.ToString();
                if (blksize > 0)
                {
                    len += 7 + 1 + blksizestr.Length + 1;
                }
                buf = new byte[len];
                int pos = 0;
                buf[pos++] = 0;
                buf[pos++] = (byte)Opcodes.OAck;
                if (timeout > 0)
                {
                    pos += Encoding.ASCII.GetBytes("timeout", 0, 7, buf, pos);
                    buf[pos++] = 0;
                    pos += Encoding.ASCII.GetBytes(timeoutstr, 0, timeoutstr.Length, buf, pos);
                    buf[pos++] = 0;
                }
                if (blksize > 0)
                {
                    pos += Encoding.ASCII.GetBytes("blksize", 0, 7, buf, pos);
                    buf[pos++] = 0;
                    pos += Encoding.ASCII.GetBytes(blksizestr, 0, blksizestr.Length, buf, pos);
                    buf[pos++] = 0;
                }
            }
            return buf;
        }
    }
}
