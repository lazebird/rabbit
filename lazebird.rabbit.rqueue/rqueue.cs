using System;
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
        public rqueue(int maxsize)
        {
            if (maxsize <= 0)
                this.maxsize = 100;
            else
                this.maxsize = maxsize;
            q = new Queue();
            l = new object();
            sc = new Semaphore(0, this.maxsize);
            sp = new Semaphore(this.maxsize, this.maxsize);
            timeout = 1000; // ms
        }
        public byte[] consume()
        {
            byte[] buf = null;
            if (!sc.WaitOne(timeout))
            {
                return buf;
            }
            lock (l)
            {
                if (q.Count > 0)
                {
                    buf = (byte[])q.Dequeue();
                    sp.Release(1);
                }
            }
            return buf;
        }
        public int produce(byte[] buf)
        {
            int len = 0;
            if (!sp.WaitOne(timeout))
            {
                return 0;
            }
            lock (l)
            {
                if (q.Count < maxsize)
                {
                    q.Enqueue(buf);
                    sc.Release(1);
                    len = buf.Length;
                }
            }
            return len;
        }
    }
}
