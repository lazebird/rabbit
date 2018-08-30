using System;
using System.Collections.Generic;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.plan
{
    public class rplan
    {
        public enum cycleunit
        {
            minute,
            hour,
            day,
            month,
            year,
            max
        }
        Action<string> log;
        public DateTime starttm;
        DateTime firetm;
        public int cycle; // second
        public cycleunit unit;
        int duration = 180; // second
        public string msg;
        Thread t;
        Thread t_ui;

        public rplan(Action<string> log, string s) // format refer to tostring()
        {
            try
            {
                string[] args = s.Split('|');
                init(log, DateTime.Parse(args[0]), int.Parse(args[1]), str2unit(args[2]), args[3]);
            }
            catch (Exception e)
            {
                log("!E: " + s + ": " + e.ToString());
            }
        }
        public rplan(Action<string> log, DateTime starttm, int cycle, cycleunit unit, string msg)
        {
            init(log, starttm, cycle, unit, msg);
        }
        void init(Action<string> log, DateTime starttm, int cycle, cycleunit unit, string msg)
        {
            this.log = log;
            this.starttm = starttm;
            this.cycle = cycle;
            this.unit = unit;
            this.msg = msg;
            t = new Thread(sched_task);
            t.IsBackground = true;
            t.Start();
        }
        bool update_firetm()
        {
            DateTime curtm = DateTime.Now;
            firetm = starttm;
            if (cycle == 0) return (starttm.CompareTo(curtm) > 0);
            switch (unit)
            {
                case cycleunit.minute:
                    while (curtm.CompareTo(firetm) > 0) firetm = firetm.AddMinutes(cycle);
                    break;
                case cycleunit.hour:
                    while (curtm.CompareTo(firetm) > 0) firetm = firetm.AddHours(cycle);
                    break;
                case cycleunit.day:
                    while (curtm.CompareTo(firetm) > 0) firetm = firetm.AddDays(cycle);
                    break;
                case cycleunit.month:
                    while (curtm.CompareTo(firetm) > 0) firetm = firetm.AddMonths(cycle);
                    break;
                case cycleunit.year:
                    while (curtm.CompareTo(firetm) > 0) firetm = firetm.AddYears(cycle);
                    break;
                default:
                    return false;
            }
            return true;
        }
        void sched_task()
        {
            try
            {
                if (!update_firetm()) return;
                while (true)
                {
                    if (DateTime.Now.CompareTo(firetm) >= 0) trigger();
                    if (!update_firetm()) return;
                    ulong firegap = (ulong)firetm.Subtract(DateTime.Now).TotalMilliseconds;
                    log("I: set <" + msg + "> fire time " + firetm.ToString() + " (" + firegap / 60000 + " min. | " + firegap / (24 * 3600000) + " d)");
                    Thread.Sleep((int)Math.Min(firegap, int.MaxValue));
                }
            }
            catch (Exception e)
            {
                log("!E: " + e.ToString());
            }
        }
        public void trigger()
        {
            log("I: <" + msg + "> triggered");
            if (t_ui != null) t_ui.Abort();
            t_ui = new Thread(() => Application.Run(new rplanform(msg, duration)));
            t_ui.IsBackground = true;
            t_ui.Start();
        }
        public void stop()
        {
            log("I: stop <" + msg + ">");
            if (t != null) t.Abort();
            t = null;
            if (t_ui != null) t_ui.Abort();
            t_ui = null;
        }
        public static List<string> cycleunitlist()
        {
            List<string> list = new List<string>();
            for (cycleunit unit = 0; unit < cycleunit.max; unit++) list.Add(unit.ToString());
            return list;
        }
        public static cycleunit str2unit(string s)
        {
            cycleunit unit;
            for (unit = 0; unit < cycleunit.max; unit++) if (unit.ToString() == s) break;
            return unit;
        }
        public override string ToString()
        {
            return starttm.ToString() + "|" + cycle + "|" + unit + "|" + msg;
        }
    }
}
