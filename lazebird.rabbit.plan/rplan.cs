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
        public TextBox tb;
        public string msg;
        public DateTime starttm;
        DateTime firetm;
        public int cycle; // second
        public cycleunit unit;
        int duration = 180; // second
        Thread t;
        Thread t_ui;

        public rplan(Action<string> log, string s, TextBox tb) // format refer to tostring()
        {
            try
            {
                string[] args = s.Split('|');
                init(log, DateTime.Parse(args[0]), int.Parse(args[1]), str2unit(args[2]), args[3], tb);
            }
            catch (Exception e)
            {
                log("!E: " + s + ": " + e.ToString());
            }
        }
        public rplan(Action<string> log, DateTime starttm, int cycle, cycleunit unit, string msg, TextBox tb)
        {
            init(log, starttm, cycle, unit, msg, tb);
        }
        void init(Action<string> log, DateTime starttm, int cycle, cycleunit unit, string msg, TextBox tb)
        {
            this.tb = tb;
            this.log = log;
            this.starttm = starttm;
            this.cycle = cycle;
            this.unit = unit;
            this.msg = msg;
            t = new Thread(sched_task);
            t.IsBackground = true;
            t.Start();
        }
        bool update_firetm(DateTime curtm)
        {
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
                if (update_firetm(DateTime.Now))
                    while (true)
                    {
                        DateTime curtm = DateTime.Now;
                        if (curtm.CompareTo(firetm) >= 0) trigger();
                        if (!update_firetm(curtm)) break;
                        ulong firegap = (ulong)firetm.Subtract(curtm).TotalMilliseconds + 1; // round up
                        tb.Text = msg + "\t @ " + firetm.ToString();// + " (" + firegap + " ms | " + firegap / 60000 + " min. | " + firegap / (24 * 3600000) + " d)";
                        Thread.Sleep((int)Math.Min(firegap, int.MaxValue));
                    }
            }
            catch (Exception) { }
            tb.Text = msg + "\t stopped ";
        }
        public void trigger()
        {
            //log("I: <" + msg + "> triggered");
            if (t_ui != null) t_ui.Abort();
            t_ui = new Thread(() => Application.Run(new rplanform(msg, duration)));
            t_ui.IsBackground = true;
            t_ui.Start();
        }
        public void stop()
        {
            tb.Text = msg + "\t stopped ";
            if (t != null) t.Abort();
            t = null;
            if (t_ui != null) t_ui.Abort();
            t_ui = null;
            tb = null;
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
