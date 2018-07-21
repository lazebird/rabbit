using lazebird.rabbit.queue;

namespace lazebird.rabbit.tftp
{
    class tftpsession
    {
        public int blkno;
        public string filename;
        public rqueue q;
        public tftpsession(int blkno, string filename, rqueue q)
        {
            this.blkno = blkno;
            this.filename = filename;
            this.q = q;
        }
    }
}
