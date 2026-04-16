using System;
using System.Text;

namespace lazebird.rabbit.tftp
{
    public class pkt
    {
        public enum Opcodes
        {
            Read = 1,
            Write = 2,
            Data = 3,
            Ack = 4,
            Error = 5,
            OAck = 6,
            ReadDir = 200,
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
            NoSuchUser = 7,
            DirNotFound = 200
        }
        public Opcodes op = 0;

        protected int parse_string(byte[] buf, int pos, ref string val)
        {
            int end = Array.FindIndex(buf, pos, (x) => x == 0);
            val = Encoding.Default.GetString(buf, pos, end - pos);
            return end + 1;
        }
        virtual public bool parse(byte[] buf) { return false; }
        virtual public bool has_opt() { return false; }
        protected int pack_string(ref byte[] buf, int pos, string s)
        {
            pos += Encoding.ASCII.GetBytes(s, 0, s.Length, buf, pos);
            buf[pos++] = 0;
            return s.Length + 1;
        }
        protected int pack_string(ref byte[] buf, int pos, string s1, string s2)
        {
            int len = pack_string(ref buf, pos, s1);
            len += pack_string(ref buf, pos + len, s2);
            return len;
        }
        virtual public byte[] pack() { return null; }
    }
}
