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
        rtftpd tftpd;
        rlog tftpdlog;
        Hashtable tftpd_dirhash;
        int curtftpd_dir = -1;
        ArrayList tftpd_dirs;
        Timer tftpd_tmr;
        Button lastclicked = null;  // donot support two different dir clicked at the same time
        void init_form_tftpd()
        {
            tftpd_dirs = new ArrayList();
            tftpd_tmr = new Timer();
            tftpd_tmr.Interval = 200; // 200ms
            tftpd_tmr.Tick += new EventHandler(tftpd_dir_tick);
            tftpd_dirhash = new Hashtable();
            tftpdlog = new rlog(tftpd_output);
            tftpd = new rtftpd(tftpd_log_func);
            tftpd_adddir.Click += new EventHandler(tftpd_adddir_click);
            tftpd_deldir.Click += new EventHandler(tftpd_deldir_click);
            tftpd_btn.Click += new EventHandler(tftpd_click);
        }
        int tftpd_log_func(int id, string msg)
        {
            return tftpdlog.write(id, msg);
        }
        void tftpd_set_dir(Button b, string path)
        {
            path = Path.GetFullPath(path);
            b.Text = (path.Length < 8) ? path : path.Substring(path.Length - 8);
            if (tftpd_dirhash.ContainsKey(b)) tftpd_dirhash.Remove(b);
            tftpd_dirhash.Add(b, path);
        }
        void tftpd_set_active(Button b)
        {
            int newidx = tftpd_dirs.IndexOf(b);
            if (newidx != curtftpd_dir && curtftpd_dir >= 0)
            {
                Button old = (Button)tftpd_dirs[curtftpd_dir];
                old.BackColor = Color.FromArgb(64, 64, 64);
            }
            b.BackColor = Color.YellowGreen;
            curtftpd_dir = newidx;
            tftpd.set_cwd((string)tftpd_dirhash[b]);
            tftpd_saveconf(); // auto save when conf changed
            //tftpd_log_func(0, "I: Activate " + tftpd_dirhash[b]);
        }
        void tftpd_dir_select(Button b)
        {
            FolderBrowserDialog dialog = new FolderBrowserDialog();
            dialog.SelectedPath = (string)tftpd_dirhash[b];
            dialog.Description = "请选择文件夹";
            if (dialog.ShowDialog() == DialogResult.OK)
            {
                tftpd_set_dir(b, dialog.SelectedPath);
            }
        }
        void tftpd_dir_click(object sender, MouseEventArgs e)
        {
            if (tftpd_tmr.Enabled)  // double click
            {
                tftpd_tmr.Stop();
                tftpd_dir_select((Button)sender);
                tftpd_set_active((Button)sender);    // auto active after select
            }
            else
            {
                lastclicked = (Button)sender;
                tftpd_tmr.Start();
            }
        }
        void tftpd_dir_tick(object sender, EventArgs e) // single click
        {
            tftpd_tmr.Stop();
            if (lastclicked == null) return;
            tftpd_set_active(lastclicked);
        }
        Button tftpd_add_dir()
        {
            Button b = new Button();
            b.BackColor = Color.FromArgb(64, 64, 64);
            b.FlatAppearance.BorderSize = 0;
            b.FlatStyle = FlatStyle.Flat;
            b.MouseDown += tftpd_dir_click;
            tftpd_fp.Controls.Add(b);
            tftpd_dirs.Add(b);
            tftpd_set_dir(b, Environment.CurrentDirectory);
            return b;
        }
        void tftpd_adddir_click(object sender, EventArgs e)
        {
            tftpd_add_dir();
        }
        void tftpd_deldir_click(object sender, EventArgs e)
        {
            if (tftpd_dirs.Count == 0) return;
            Button b = (Button)tftpd_dirs[tftpd_dirs.Count - 1];
            tftpd_fp.Controls.Remove(b);
            tftpd_dirs.Remove(b);
            tftpd_dirhash.Remove(b);
            curtftpd_dir = -1;  // reset cur index, to avoid problems
        }
        void tftpd_click(object sender, EventArgs e)
        {
            if (((Button)btnhash["tftpd_btn"]).Text == Language.trans("开始"))
            {
                tftpdlog.clear();
                ((Button)btnhash["tftpd_btn"]).Text = Language.trans("停止");
                tftpd.start(69, int.Parse(((TextBox)texthash["tftpd_timeout"]).Text), int.Parse(((TextBox)texthash["tftpd_retry"]).Text));
            }
            else
            {
                ((Button)btnhash["tftpd_btn"]).Text = Language.trans("开始");
                tftpd.stop();
            }
            saveconf();
        }
        void tftpd_readconf()
        {
            string[] dirs = rconf.get("tftpd_dirs").Split(';');
            foreach (string path in dirs)
            {
                if (path != "")
                    tftpd_set_dir(tftpd_add_dir(), path);
            }
            int index = int.Parse(rconf.get("tftpd_dir_index"));
            if (index >= 0 && index < tftpd_dirs.Count)
                tftpd_set_active((Button)tftpd_dirs[index]);
        }
        void tftpd_saveconf()
        {
            if (onloading) return;
            rconf.set("tftpd_dir_index", curtftpd_dir.ToString());
            string dirs = "";
            foreach (string path in tftpd_dirhash.Values)
            {
                dirs += path + ";";
            }
            rconf.set("tftpd_dirs", dirs);
        }
    }
}
