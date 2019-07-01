using lazebird.rabbit.common;
using lazebird.rabbit.plan;
using System;
using System.Collections;
using System.Windows.Forms;
using static lazebird.rabbit.plan.rplan;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rpanel plan_panel;
        rlog planlog;
        Hashtable plan_tbhash;
        Hashtable plan_msghash;
        bool plan_override;
        void init_form_plan()
        {
            plan_panel = new rpanel(fp_plan);
            planlog = new rlog(lb_plan);
            plan_tbhash = new Hashtable();
            plan_msghash = new Hashtable();
            btn_planadd.Click += new EventHandler(plan_add_click);
            btn_plandel.Click += new EventHandler(plan_del_click);
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
            int cycle = 0;
            int.TryParse(text_plancycle.Text, out cycle);
            cycleunit unit = str2unit(cb_planunit.Text);
            string msg = text_planmsg.Text.Trim();
            if (string.IsNullOrWhiteSpace(msg))
            {
                plan_log_func("!E: msg cannot be null or empty!");
                return null;
            }
            if (plan_msghash.ContainsKey(msg))
            {
                if (!plan_override)
                {
                    plan_log_func("!E: <" + msg + "> already exists!");
                    return null;
                }
                plan_del(msg);
            }
            return new rplan(plan_log_func, starttm, cycle, unit, msg, plan_panel.add(msg, null, plan_click));
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
            if (p == null) return;
            plan_msghash.Add(p.msg, p.tb);
            plan_tbhash.Add(p.tb, p);
        }
        void plan_del(string msg)
        {
            if (!plan_msghash.ContainsKey(text_planmsg.Text))
            {
                plan_log_func("!E: <" + msg + "> not found!");
                return;
            }
            TextBox tb = (TextBox)plan_msghash[text_planmsg.Text];
            rplan p = (rplan)plan_tbhash[tb];
            p.stop();
            plan_msghash.Remove(text_planmsg.Text);
            plan_tbhash.Remove(tb);
            plan_panel.del(tb);
        }
        void plan_parse_args()
        {
            Hashtable opts = ropt.parse_opts(text_planopt.Text);
            if (opts.ContainsKey("override")) bool.TryParse((string)opts["override"], out plan_override);
        }
        void plan_add_click(object sender, EventArgs e)
        {
            plan_parse_args();
            plan_add(ui2plan());
        }
        void plan_del_click(object sender, EventArgs e)
        {
            plan_del(text_planmsg.Text);
        }
        void plan_readconf()
        {
            string[] cfgs = cfg.getstr("plans").Split(';');
            foreach (string cfg in cfgs)
                if (cfg.Length > 0) plan_add(new rplan(plan_log_func, cfg, plan_panel.add(cfg, null, plan_click)));
        }
        void plan_saveconf()
        {
            if (onloading) return;
            string cfgs = "";
            foreach (rplan p in plan_tbhash.Values)
                cfgs += p.ToString() + ";";
            cfg.set("plans", cfgs);
        }
    }
}
