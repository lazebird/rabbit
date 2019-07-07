using lazebird.rabbit.common;
using lazebird.rabbit.tftp;
using System;
using System.Collections;
using System.Drawing;
using System.IO;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rpanel tftpd_dpanel;
        rtftpd tftpd;
        rpanel tftpdlog;
        Hashtable tftpd_dirhash;
        int curtftpd_dir = -1;
        ArrayList tftpd_dirs;
        Timer tftpd_tmr;
        TextBox lastclicked = null;  // donot support two different dir clicked at the same time
        string lasttftpddir = Environment.CurrentDirectory;
        void init_form_tftpd()
        {
            tftpd_dpanel = new rpanel(fp_tftpd_dir);
            tftpdlog = new rpanel(fp_tftpd_log);
            tftpd_dirs = new ArrayList();
            tftpd_tmr = new Timer();
            tftpd_tmr.Interval = 200; // 200ms
            tftpd_tmr.Tick += new EventHandler(tftpd_dir_tick);
            tftpd_dirhash = new Hashtable();
            tftpd = new rtftpd(tftpd_log_func);
            tftpd_adddir.Click += tftpd_adddir_click;
            tftpd_deldir.Click += tftpd_deldir_click;
            tftpd_btn.Click += tftpd_click;
            fp_tftpd_dir.AutoScroll = true;
            key.bind(3, Keys.Enter, tftpd_click);
        }
        int tftpd_log_func(int id, string msg)
        {
            return tftpdlog.write(id, msg);
        }
        void tftpd_set_dir(TextBox b, string path)
        {
            path = Path.GetFullPath(path);
            b.Text = path;
            if (tftpd_dirhash.ContainsKey(b)) tftpd_dirhash.Remove(b);
            tftpd_dirhash.Add(b, path);
        }
        void tftpd_active_dir(TextBox tb)
        {
            int newidx = tftpd_dirs.IndexOf(tb);
            if (newidx != curtftpd_dir && curtftpd_dir >= 0)
            {
                TextBox old = (TextBox)tftpd_dirs[curtftpd_dir];
                old.BackColor = fp_tftpd_dir.BackColor;
            }
            tb.BackColor = Color.YellowGreen;
            curtftpd_dir = newidx;
            tftpd.set_cwd((string)tftpd_dirhash[tb]);
            //tftpd_log_func(0, "I: Activate " + tftpd_dirhash[b]);
        }
        TextBox tftpd_dir_select(TextBox tb)
        {
            SaveFileDialog dialog = new SaveFileDialog();
            dialog.InitialDirectory = (string)tftpd_dirhash[tb];
            dialog.RestoreDirectory = true;
            dialog.FileName = "Choose Here";
            if (dialog.ShowDialog() == DialogResult.OK)
            {
                lasttftpddir = Path.GetDirectoryName(dialog.FileName);
                tftpd_set_dir(tb, lasttftpddir);
            }
            return tb;
        }
        void tftpd_dir_click(object sender, MouseEventArgs e)
        {
            if (tftpd_tmr.Enabled)  // double click
            {
                tftpd_tmr.Stop();
                tftpd_dir_select((TextBox)sender);
                tftpd_active_dir((TextBox)sender);    // auto active after select
            }
            else
            {
                lastclicked = (TextBox)sender;
                tftpd_tmr.Start();
            }
        }
        void tftpd_dir_tick(object sender, EventArgs e) // single click
        {
            tftpd_tmr.Stop();
            if (lastclicked == null) return;
            tftpd_active_dir(lastclicked);
        }
        TextBox tftpd_add_dir()
        {
            TextBox tb = tftpd_dpanel.add("", tftpd_dir_click, null);
            tftpd_dirs.Add(tb);
            tftpd_set_dir(tb, lasttftpddir);
            return tb;
        }
        void tftpd_del_dir(TextBox tb)
        {
            tftpd_dirs.Remove(tb);
            tftpd_dirhash.Remove(tb);
            tftpd_dpanel.del(tb);
        }
        void tftpd_adddir_click(object sender, EventArgs e)
        {
            tftpd_active_dir(tftpd_dir_select(tftpd_add_dir()));    // auto active after select
        }
        void tftpd_deldir_click(object sender, EventArgs e)
        {
            if (curtftpd_dir < 0 || curtftpd_dir >= tftpd_dirs.Count) return;
            tftpd_del_dir((TextBox)tftpd_dirs[curtftpd_dir]);
            curtftpd_dir = -1;  // reset cur index, to avoid problems
            if (tftpd_dirs.Count > 0) tftpd_active_dir((TextBox)tftpd_dirs[tftpd_dirs.Count - 1]);
        }
        void tftpd_click(object sender, EventArgs evt)
        {
            try
            {
                if (cfg.getstr("tftpd_btn") == Language.trans("开始"))
                {
                    tftpdlog.clear();
                    cfg.set("tftpd_btn", Language.trans("停止"));
                    tftpd.start(69, ropt.parse_opts(text_tftpdopt.Text));
                }
                else
                {
                    cfg.set("tftpd_btn", Language.trans("开始"));
                    tftpd.stop();
                }
            }
            catch (Exception e)
            {
                tftpd_log_func(-1, "!E: " + e.ToString());
            }
        }
        void tftpd_conf_set(string name, string val)
        {
            if (name == "tftpd_dirs")
            {
                string[] dirs = val.Split(';');
                foreach (string path in dirs)
                {
                    if (path != "") tftpd_set_dir(tftpd_add_dir(), path);
                }
                return;
            }
            int index = int.Parse(val);
            if (index >= 0 && index < tftpd_dirs.Count) tftpd_active_dir((TextBox)tftpd_dirs[index]);
        }
        string tftpd_conf_get(string name)
        {
            if (name == "tftpd_dir_index")
            {
                return curtftpd_dir.ToString();
            }
            string dirs = "";
            for (int i = 0; i < tftpd_dirs.Count; i++)
            {
                dirs += tftpd_dirhash[tftpd_dirs[i]] + ";";
            }
            return dirs;
        }
    }
}
