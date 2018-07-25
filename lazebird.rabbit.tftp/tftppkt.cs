using System;
using System.Text;

namespace lazebird.rabbit.tftp
{
    public class tftppkt
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
        public Opcodes op = 0;
        public string filename = null;
        public string filepath = null;
        public string mode = null;
        public int blksize = 0;
        public int timeout = 0;
        public int blkno = 0;
        public byte[] data = null;
        public Errcodes errno = 0;
        public string errmsg = null;
        public tftppkt()
        {
        }
        public tftppkt(Opcodes op, string filename, string mode) : this()    // req
        {
            this.op = op;
            this.filename = filename;
            this.mode = mode;
        }
        public tftppkt(Opcodes op, string filename, string mode, int timeout, int blksize) : this(op, filename, mode)    // req + opt
        {
            this.timeout = timeout;
            this.blksize = blksize;
        }
        public tftppkt(Opcodes op, int blkno, byte[] data) : this()   // data
        {
            this.op = op;
            this.blkno = blkno;
            this.data = data;
        }
        public tftppkt(Opcodes op, int blkno) : this()    // ack
        {
            this.op = op;
            this.blkno = blkno;
        }
        public tftppkt(Opcodes op, Errcodes errno, string errmsg) : this()   // err
        {
            this.op = op;
            this.errno = errno;
            this.errmsg = errmsg;
        }
        public tftppkt(Opcodes op, int timeout, int blksize) : this()    // oack
        {
            this.op = op;
            this.timeout = timeout;
            this.blksize = blksize;
        }
        int parse_string(byte[] buf, int pos, ref string val)
        {
            int end = Array.FindIndex(buf, pos, (x) => x == 0);
            val = Encoding.Default.GetString(buf, pos, end - pos);
            return end + 1;
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
            try
            {
                this.timeout = this.blksize = 0; // reset 
                op = (Opcodes)buf[1];
                if (op == Opcodes.Read || op == Opcodes.Write)
                {
                    int pos = 2;
                    pos = parse_string(buf, pos, ref filename);
                    filepath = "/" + filename;
                    pos = parse_string(buf, pos, ref mode);
                    string optname = null;
                    string optval = null;
                    while (pos < buf.Length)
                    {
                        pos = parse_string(buf, pos, ref optname);
                        pos = parse_string(buf, pos, ref optval);
                        parse_opt(optname, optval);
                    }
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
                else if (op == Opcodes.OAck)
                {
                    int pos = 2;
                    string optname = null;
                    string optval = null;
                    while (pos < buf.Length)
                    {
                        pos = parse_string(buf, pos, ref optname);
                        pos = parse_string(buf, pos, ref optval);
                        parse_opt(optname, optval);
                    }
                }
                else
                    return false;
                return true;
            }
            catch (Exception e)
            {
                errmsg = e.ToString();
                return false;
            }
        }
        int pack_string(ref byte[] buf, int pos, string s)
        {
            pos += Encoding.ASCII.GetBytes(s, 0, s.Length, buf, pos);
            buf[pos++] = 0;
            return s.Length + 1;
        }
        int pack_string(ref byte[] buf, int pos, string s1, string s2)
        {
            int len = pack_string(ref buf, pos, s1);
            len += pack_string(ref buf, pos + len, s2);
            return len;
        }
        public byte[] pack()
        {
            byte[] buf = null;
            int pos = 0;
            if (op == Opcodes.Read || op == Opcodes.Write)
            {
                int len = mode.Length + filename.Length + 4;
                string timeoutstr = timeout.ToString();
                if (timeout > 0) len += 7 + 1 + timeoutstr.Length + 1;
                string blksizestr = blksize.ToString();
                if (blksize > 0) len += 7 + 1 + blksizestr.Length + 1;
                buf = new byte[len];
                buf[pos++] = 0;
                buf[pos++] = (byte)op;
                pos += pack_string(ref buf, pos, filename);
                pos += pack_string(ref buf, pos, mode);
                if (timeout > 0) pos += pack_string(ref buf, pos, "timeout", timeoutstr);
                if (blksize > 0) pos += pack_string(ref buf, pos, "blksize", blksizestr);
            }
            else if (op == Opcodes.Data)
            {
                buf = new byte[4 + data.Length];
                buf[pos++] = 0;
                buf[pos++] = (byte)Opcodes.Data;
                buf[pos++] = (byte)((blkno >> 8) & 0xff);
                buf[pos++] = (byte)(blkno & 0xff);
                Array.Copy(data, 0, buf, pos++, data.Length);
            }
            else if (op == Opcodes.Ack)
            {
                buf = new byte[4];
                buf[pos++] = 0;
                buf[pos++] = (byte)Opcodes.Ack;
                buf[pos++] = (byte)((blkno >> 8) & 0xff);
                buf[pos++] = (byte)(blkno & 0xff);
            }
            else if (op == Opcodes.Error)
            {
                buf = new byte[5 + errmsg.Length];
                buf[pos++] = 0;
                buf[pos++] = (byte)Opcodes.Error;
                buf[pos++] = 0;
                buf[pos++] = (byte)errno;
                pos += pack_string(ref buf, pos, errmsg);
            }
            else if (op == Opcodes.OAck)
            {
                int len = 2;
                string timeoutstr = timeout.ToString();
                if (timeout > 0) len += 7 + 1 + timeoutstr.Length + 1;
                string blksizestr = blksize.ToString();
                if (blksize > 0) len += 7 + 1 + blksizestr.Length + 1;
                buf = new byte[len];
                buf[pos++] = 0;
                buf[pos++] = (byte)Opcodes.OAck;
                if (timeout > 0) pos += pack_string(ref buf, pos, "timeout", timeoutstr);
                if (blksize > 0) pos += pack_string(ref buf, pos, "blksize", blksizestr);
            }
            return buf;
        }
    }
}
