using System;

namespace lazebird.rabbit.tftp
{
    class oack_pkt : pkt
    {
        public int timeout = 0;
        public int blksize = 0;

        public oack_pkt()
        {
            op = Opcodes.OAck;
        }

        public oack_pkt(int timeout, int blksize) : this()
        {
            this.timeout = timeout;
            this.blksize = blksize;
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
        override public bool parse(byte[] buf)
        {
            if ((Opcodes)buf[1] != op) return false;
            int pos = 2;
            string optname = null;
            string optval = null;
            while (pos < buf.Length)
            {
                pos = parse_string(buf, pos, ref optname);
                pos = parse_string(buf, pos, ref optval);
                parse_opt(optname, optval);
            }
            return true;
        }
        override public byte[] pack()
        {
            byte[] buf = null;
            int pos = 0;
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
            return buf;
        }
    }
}
