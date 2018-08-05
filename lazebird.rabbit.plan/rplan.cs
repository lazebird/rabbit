using System;
using System.Collections.Generic;
using System.Drawing;
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
            if (update_firetm())
            {
                t = new Thread(sched_task);
                t.IsBackground = true;
                t.Start();
            }
        }
        bool update_firetm()
        {
            DateTime curtm = DateTime.Now;
            if (starttm.CompareTo(curtm) > 0)
            {
                firetm = starttm;
                return true;
            }
            if (cycle == 0) return false;
            switch (unit)
            {
                case cycleunit.minute:
                    firetm = starttm.AddMinutes(cycle);
                    while (curtm.CompareTo(firetm) > 0) firetm = firetm.AddMinutes(cycle);
                    break;
                case cycleunit.hour:
                    firetm = starttm.AddHours(cycle);
                    while (curtm.CompareTo(firetm) > 0) firetm = firetm.AddHours(cycle);
                    break;
                case cycleunit.day:
                    firetm = starttm.AddDays(cycle);
                    while (curtm.CompareTo(firetm) > 0) firetm = firetm.AddDays(cycle);
                    break;
                case cycleunit.month:
                    firetm = starttm.AddMonths(cycle);
                    while (curtm.CompareTo(firetm) > 0) firetm = firetm.AddMonths(cycle);
                    break;
                case cycleunit.year:
                    firetm = starttm.AddYears(cycle);
                    while (curtm.CompareTo(firetm) > 0) firetm = firetm.AddYears(cycle);
                    break;
                default:
                    return false;
            }
            log("I: set <" + msg + "> fire time " + firetm.ToString());
            return true;
        }
        void sched_task()
        {
            while (true)
            {
                if (DateTime.Now.CompareTo(firetm) >= 0)
                {
                    trigger();
                    if (!update_firetm()) break;
                }
                else
                    Thread.Sleep((int)firetm.Subtract(DateTime.Now).TotalMilliseconds);
            }
            try
            {
            }
            catch (Exception e)
            {
                log("!E: " + e.ToString());
            }
        }
        int loop;
        void form_click(object sender, EventArgs e)
        {
            loop = 0;
        }
        void form_load(object sender, EventArgs e)
        {
            Form f = (Form)sender;
            Label l = new Label();
            f.Controls.Add(l);
            Thread t1 = new Thread(() => label_task(f, l));
            t1.IsBackground = true;
            t1.Start();
        }
        void label_task(Form f, Label l)
        {
            l.Location = new Point(10, f.Height / 2);
            l.Width = f.Width - 20;
            l.TextAlign = ContentAlignment.MiddleCenter;
            l.Font = new Font(l.Font.FontFamily, 12, l.Font.Style);
            l.BackColor = Color.Black;
            l.ForeColor = Color.White;
            loop = duration;
            while (loop-- > 0)
            {
                l.Text = msg + " (" + loop + ")";
                Thread.Sleep(1000); // update progress per second
            }
            l.Dispose();
            f.Dispose();
            t_ui = null;
        }
        void form_task()
        {
            Form f = new Form();
            f.FormBorderStyle = FormBorderStyle.None;
            f.WindowState = FormWindowState.Maximized;
            f.BackColor = Color.Black;
            f.TopMost = true;
            f.Text = msg + " " + DateTime.Now;
            f.DoubleClick += new EventHandler(form_click);
            f.Load += new EventHandler(form_load);
            Application.Run(f);
        }
        public void trigger()
        {
            log("I: trigger " + msg);
            if (t_ui != null) return;
            t_ui = new Thread(form_task);
            t_ui.IsBackground = true;
            t_ui.Start();
        }
        public void stop()
        {
            if (t != null) t.Abort();
            t = null;
            if (t_ui != null) t.Abort();
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
