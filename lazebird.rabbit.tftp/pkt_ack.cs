namespace lazebird.rabbit.tftp
{
    class pkt_ack : pkt
    {
        public int blkno = 0;

        public pkt_ack()
        {
            op = Opcodes.Ack;
        }

        public pkt_ack(int blkno) : this()
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
