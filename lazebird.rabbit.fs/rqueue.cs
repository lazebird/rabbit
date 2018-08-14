using System;
using System.Collections;
using System.Threading;

namespace lazebird.rabbit.fs
{
    public class rqueue : IDisposable
    {
        Queue q;
        object l;
        Semaphore sc = null;
        Semaphore sp = null;
        int maxsize = 100;
        int timeout = 1000;
        bool stop_flag = false;
        public int stat_produce = 0;
        public int stat_consume = 0;
        public rqueue()
        {
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
                        stat_consume++;
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
                        stat_produce++;
                        sc.Release(1);
                        len = buf.Length;
                    }
            return len;
        }
        public void stop()
        {
            stop_flag = true;
        }
        public bool has_stopped()
        {
            return stop_flag;
        }
        protected virtual void Dispose(bool disposing)
        {
            if (sc != null) sc.Dispose();
            if (sp != null) sp.Dispose();
            sc = null;
            sp = null;
            if (!disposing) return;
            q = null;
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
