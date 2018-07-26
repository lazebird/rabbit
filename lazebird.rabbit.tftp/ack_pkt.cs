namespace lazebird.rabbit.tftp
{
    class ack_pkt : pkt
    {
        public int blkno = 0;

        public ack_pkt()
        {
            op = Opcodes.Ack;
        }

        public ack_pkt(int blkno) : this()
        {
            this.blkno = blkno;
        }

        override public bool parse(byte[] buf)
        {
            if ((Opcodes)buf[1] != op) return false;
            blkno = buf[2] << 8 | buf[3];
            return true;
        }
        override public byte[] pack()
        {
            byte[] buf = null;
            int pos = 0;
            buf = new byte[4];
            buf[pos++] = 0;
            buf[pos++] = (byte)Opcodes.Ack;
            buf[pos++] = (byte)((blkno >> 8) & 0xff);
            buf[pos++] = (byte)(blkno & 0xff);
            return buf;
        }
    }
}
