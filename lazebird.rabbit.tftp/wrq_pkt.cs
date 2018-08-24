namespace lazebird.rabbit.tftp
{
    class wrq_pkt : pkt
    {
        public string filename = null;
        public string filepath = null;
        public string mode = null;
        public int timeout = 0;
        public int blksize = 0;

        public wrq_pkt()
        {
            op = Opcodes.Write;
        }

        public wrq_pkt(string filename, string mode, int timeout, int blksize) : this()
        {
            this.filename = filename;
            this.filepath = "/" + filename;
            this.mode = mode;
            this.timeout = timeout > 0 ? timeout : 2;
            this.blksize = blksize > 0 ? blksize : 512;
        }
        override public bool has_opt() { return (timeout != 0 || blksize != 0); }
        bool parse_opt(string name, string val)
        {
            if (name == "timeout") return int.TryParse(val, out timeout);
            else if (name == "blksize") return int.TryParse(val, out blksize);
            else return false;
        }
        override public bool parse(byte[] buf)
        {
            if ((Opcodes)buf[1] != op) return false;
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
            return true;
        }
        override public byte[] pack()
        {
            byte[] buf = null;
            int pos = 0;
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
            return buf;
        }
    }
}
