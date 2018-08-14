using lazebird.rabbit.common;
using lazebird.rabbit.plan;
using System;
using System.Collections;
using System.Drawing;
using System.Windows.Forms;
using static lazebird.rabbit.plan.rplan;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        Hashtable plan_tbhash;
        Hashtable plan_msghash;
        rlog planlog;
        void init_form_plan()
        {
            plan_tbhash = new Hashtable();
            plan_msghash = new Hashtable();
            planlog = new rlog(plan_output);
            btn_planadd.Click += new EventHandler(plan_add_click);
            btn_plandel.Click += new EventHandler(plan_del_click);
            fp_plan.AutoScroll = true;
            cb_planunit.DataSource = cycleunitlist();
            cb_planunit.Text = cycleunit.minute.ToString();
        }
        void plan_log_func(string msg)
        {
            planlog.write(msg);
        }
        rplan ui2plan()
        {
            DateTime starttm = DateTime.Parse(dt1_plan.Text + " " + dt2_plan.Text);
            int cycle = int.Parse(text_plancycle.Text);
            cycleunit unit = str2unit(cb_planunit.Text);
            string msg = text_planmsg.Text;
            if (msg == "")
            {
                plan_log_func("I: msg cannot be null or empty!");
                return null;
            }
            return new rplan(plan_log_func, starttm, cycle, unit, msg);
        }
        void plan2ui(rplan p)
        {
            dt1_plan.Text = p.starttm.ToString("D");
            dt2_plan.Text = p.starttm.ToString("t");
            text_plancycle.Text = p.cycle.ToString();
            cb_planunit.Text = p.unit.ToString();
            text_planmsg.Text = p.msg;
        }
        void plan_click(object sender, EventArgs e)
        {
            if (!plan_tbhash.ContainsKey(sender)) return;
            rplan p = (rplan)plan_tbhash[sender];
            plan2ui(p);
            //p.trigger(); // for test
        }
        void plan_add(rplan p)
        {
            if (p == null || plan_msghash.ContainsKey(p.msg)) return;
            TextBox tb = new TextBox();
            tb.ReadOnly = true;
            tb.BackColor = Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            tb.BorderStyle = BorderStyle.None;
            tb.ForeColor = Color.White;
            tb.Text = p.msg;
            tb.Width = 760;
            //tb.Anchor = (AnchorStyles.Top | AnchorStyles.Left | AnchorStyles.Right);
            tb.DoubleClick += new EventHandler(plan_click);
            fp_plan.Controls.Add(tb);
            plan_msghash.Add(p.msg, tb);
            plan_tbhash.Add(tb, p);
        }
        void plan_del(string msg)
        {
            if (!plan_msghash.ContainsKey(text_planmsg.Text)) return;
            TextBox tb = (TextBox)plan_msghash[text_planmsg.Text];
            rplan p = (rplan)plan_tbhash[tb];
            p.stop();
            fp_plan.Controls.Remove(tb);
            plan_msghash.Remove(text_planmsg.Text);
            plan_tbhash.Remove(tb);
        }
        void plan_add_click(object sender, EventArgs e)
        {
            plan_add(ui2plan());
            saveconf();
        }
        void plan_del_click(object sender, EventArgs e)
        {
            plan_del(text_planmsg.Text);
            saveconf();
        }
        void plan_readconf()
        {
            string[] cfgs = rconf.get("plans").Split(';');
            foreach (string cfg in cfgs)
                if (cfg.Length > 0) plan_add(new rplan(plan_log_func, cfg));
        }
        void plan_saveconf()
        {
            if (onloading) return;
            string cfgs = "";
            foreach (rplan p in plan_tbhash.Values)
                cfgs += p.ToString() + ";";
            rconf.set("plans", cfgs);
        }
    }
}
