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
        public rqueue() : this(100, 1000) { }
        public rqueue(int maxsize) : this(maxsize, 1000) { }
        public rqueue(int maxsize, int timeout)
        {
            this.maxsize = maxsize;
            this.timeout = timeout;
            q = new Queue();
            l = new object();
            sc = new Semaphore(0, this.maxsize);
            sp = new Semaphore(this.maxsize, this.maxsize);
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
