using System.Collections;
using System.Threading;

namespace lazebird.rabbit.queue
{
    public class rqueue
    {
        Queue q;
        object l;
        Semaphore sc;
        Semaphore sp;
        int maxsize;
        int timeout;
        public rqueue()
        {
            this.maxsize = 100;
            this.timeout = 1000;
            q = new Queue();
            l = new object();
            sc = new Semaphore(0, this.maxsize);
            sp = new Semaphore(this.maxsize, this.maxsize);
        }
        public rqueue(int maxsize) : this()
        {
            this.maxsize = maxsize;
        }
        public rqueue(int maxsize, int timeout) : this()
        {
            this.maxsize = maxsize;
            this.timeout = timeout;
        }
        public byte[] consume()
        {
            byte[] buf = null;
            if (sc.WaitOne(timeout))
                lock (l)
                    if (q.Count > 0)
                    {
                        buf = (byte[])q.Dequeue();
                        sp.Release(1);
                    }
            return buf;
        }
        public int produce(byte[] buf)
        {
            int len = 0;
            if (sp.WaitOne(timeout))
                lock (l)
                    if (q.Count < maxsize)
                    {
                        q.Enqueue(buf);
                        sc.Release(1);
                        len = buf.Length;
                    }
            return len;
        }
    }
}
