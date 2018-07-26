using System.Text;

namespace lazebird.rabbit.tftp
{
    class err_pkt : pkt
    {
        public Errcodes errno = 0;
        public string errmsg = null;

        public err_pkt()
        {
            op = Opcodes.Error;
        }
        public err_pkt(Errcodes errno, string errmsg) : this()
        {
            this.errno = errno;
            this.errmsg = errmsg;
        }

        override public bool parse(byte[] buf)
        {
            if ((Opcodes)buf[1] != op) return false;
            errno = (Errcodes)(buf[2] << 8 | buf[3]);
            errmsg = Encoding.Default.GetString(buf, 4, buf.Length - 5);
            return true;
        }
        override public byte[] pack()
        {
            byte[] buf = null;
            int pos = 0;
            buf = new byte[5 + errmsg.Length];
            buf[pos++] = 0;
            buf[pos++] = (byte)Opcodes.Error;
            buf[pos++] = 0;
            buf[pos++] = (byte)errno;
            pos += pack_string(ref buf, pos, errmsg);
            return buf;
        }
    }
}
