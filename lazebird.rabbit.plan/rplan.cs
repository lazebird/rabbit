using System;
using System.Threading;

namespace lazebird.rabbit.plan
{
    public class rplan
    {
        Action<string> log;
        DateTime firetm;
        int cycle; // second
        int duration; // second
        string msg;
        Thread t;
        public rplan(Action<string> log, DateTime firetm, int cycle, int duration, string msg)
        {
            this.log = log;
            this.firetm = firetm;
            this.cycle = cycle;
            this.duration = duration;
            this.msg = msg;
            t = new Thread(sched_task);
            t.IsBackground = true;
            t.Start();
        }
        void sched_task()
        {
            try
            {
                while (true)
                {
                    DateTime curtm = DateTime.Now;
                    if (curtm.CompareTo(firetm) >= 0)
                    {
                        trigger();
                        if (cycle <= 0) return;
                        firetm = curtm.AddSeconds(cycle);
                    }
                    else
                        Thread.Sleep((int)curtm.Subtract(firetm).TotalMilliseconds);
                }
            }
            catch (Exception e)
            {
                log("!E: " + e.ToString());
            }
        }
        void trigger()
        {
            log("I: trigger " + msg);
            int loop = duration;
            while (loop-- > 0)
            {
                Thread.Sleep(1000);
            }
        }
        public void stop()
        {
            t.Abort();
            t = null;
        }
    }
}
