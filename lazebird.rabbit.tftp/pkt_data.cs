using System;

namespace lazebird.rabbit.tftp
{
    class pkt_data : pkt
    {
        public int blkno = 0;
        public byte[] data = null;

        public pkt_data()
        {
            op = Opcodes.Data;
        }

        public pkt_data(int blkno, byte[] data) : this()
        {
            this.blkno = blkno;
            this.data = data;
        }

        override public bool parse(byte[] buf)
        {
            if ((Opcodes)buf[1] != op) return false;
            blkno = buf[2] << 8 | buf[3];
            data = new byte[buf.Length - 4];
            Array.Copy(buf, 4, data, 0, buf.Length - 4);
            return true;
        }
        override public byte[] pack()
        {
            byte[] buf = null;
            int pos = 0;
            buf = new byte[4 + data.Length];
            buf[pos++] = 0;
            buf[pos++] = (byte)Opcodes.Data;
            buf[pos++] = (byte)((blkno >> 8) & 0xff);
            buf[pos++] = (byte)(blkno & 0xff);
            Array.Copy(data, 0, buf, pos++, data.Length);
            return buf;
        }
    }
}
