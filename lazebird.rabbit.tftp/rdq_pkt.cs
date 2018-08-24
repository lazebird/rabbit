namespace lazebird.rabbit.tftp
{
    class rdq_pkt : pkt
    {
        public string dirname = null;

        public rdq_pkt()
        {
            op = Opcodes.ReadDir;
        }

        public rdq_pkt(string dirname) : this()
        {
            this.dirname = dirname;
        }
        override public bool parse(byte[] buf)
        {
            if ((Opcodes)buf[1] != op) return false;
            int pos = 2;
            pos = parse_string(buf, pos, ref dirname);
            return true;
        }
        override public byte[] pack()
        {
            byte[] buf = null;
            int pos = 0;
            int len = dirname.Length + 4;
            buf = new byte[len];
            buf[pos++] = 0;
            buf[pos++] = (byte)op;
            pos += pack_string(ref buf, pos, dirname);
            return buf;
        }
    }
}
