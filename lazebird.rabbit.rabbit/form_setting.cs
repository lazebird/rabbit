using lazebird.rabbit.common;
using lazebird.vgen.ver;
using System;
using System.Collections.Generic;
using System.IO;
using System.Net;
using System.Reflection;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rlog setlog;
        Version pver;
        string prjurl = "https://code.aliyun.com/lazebird/rabbit/tree/master/release";
        string profileuri = @"%userprofile%\appdata\local";
        string helpurl = "https://code.aliyun.com/lazebird/rabbit/blob/master/doc/manual.md";
        void init_form_setting()
        {
            setlog = new rlog(setting_output);
            pver = Assembly.GetExecutingAssembly().GetName().Version;
            link_ver.Text = pver.ToString() + " (" + appver.v.ToString() + ")";
            link_ver.LinkClicked += ver_click;
            link_prj.LinkClicked += prj_click;
            link_prof.LinkClicked += profile_click;
            link_help.LinkClicked += help_click;
            List<string> list = new List<string>();
            list.Add("System");
            list.Add("English");
            list.Add("中文");
            lang_cb.DataSource = list;
            lang_cb.Text = Language.Getlang();
            lang_cb.SelectedIndexChanged += lang_opt_SelectedIndexChanged;
            setlog.write("Language: " + Language.Getlang());
            if (File.Exists(upgrade.scriptpath)) File.Delete(upgrade.scriptpath);
            init_systray();
            cb_top.CheckedChanged += top_click;
            cb_autostart.CheckedChanged += autostart_click;
            if (sh.autostart_exist()) cb_autostart.Checked = true;
        }
        void lang_opt_SelectedIndexChanged(object sender, EventArgs e)
        {
            // 将被选中的项目强制转换为MyItem
            string lang = lang_cb.SelectedItem as string;
            if (lang_cb.Text == "System")
            {
                rconf.set("lang", "");
            }
            else
            {
                rconf.set("lang", lang_cb.Text);
            }
            setlog.write("Set Language: " + lang_cb.Text);
            setlog.write("Restart App to take effect!");
        }
        void prj_click(object sender, LinkLabelLinkClickedEventArgs e)
        {
            System.Diagnostics.Process.Start(prjurl);
        }
        void profile_click(object sender, LinkLabelLinkClickedEventArgs e)
        {
            string path = Environment.ExpandEnvironmentVariables(profileuri);
            System.Diagnostics.Process.Start(path);
        }
        void help_click(object sender, LinkLabelLinkClickedEventArgs e)
        {
            System.Diagnostics.Process.Start(helpurl);
        }
        string download2str(string uri)
        {
            WebClient wc = new WebClient();
            string data = wc.DownloadString(uri);
            wc.Dispose();
            return data;
        }
        void download2file(string uri, string filepath)
        {
            WebClient wc = new WebClient();
            wc.DownloadFile(uri, filepath);
            wc.Dispose();
        }
        string versionuri = "https://code.aliyun.com/lazebird/rabbit/raw/master/release/version.txt";
        string binaryuri = "https://code.aliyun.com/lazebird/rabbit/raw/master/release/sRabbit.exe";
        void ver_check()
        {
            try
            {
                string s = download2str(versionuri);
                verinfo v = new verinfo(s);
                if (string.Compare(v.HeadShaShort, appver.v.HeadShaShort) == 0)
                {
                    setlog.write("Version is up to date!");
                    return;
                }
                MessageBoxButtons mbox = MessageBoxButtons.YesNo;
                string newbinpath = Application.ExecutablePath + ".new";
                DialogResult res = MessageBox.Show("New release will be saved to " + newbinpath, "Rabbit Upgrade", mbox);
                if (res == DialogResult.Yes)
                {
                    download2file(binaryuri, newbinpath);
                    MessageBoxButtons mbox2 = MessageBoxButtons.YesNo;
                    DialogResult res2 = MessageBox.Show("Restart to upgrade now?", "Rabbit Upgrade", mbox2);
                    if (res2 == DialogResult.Yes)
                    {
                        upgrade upg = new upgrade(Application.ExecutablePath);
                        upg.run();
                        Application.Exit();
                    }
                    else setlog.write("New release saved to " + newbinpath);

                }
                else setlog.write("The newest version is " + v.ToString() + ", click the homepage to download it.");

            }
            catch (Exception e)
            {
                setlog.write("!E: " + e.ToString());
            }
        }
        void ver_click(object sender, LinkLabelLinkClickedEventArgs evt)
        {
            Clipboard.SetDataObject(link_ver.Text);
            Thread t = new Thread(ver_check);
            t.IsBackground = true;
            t.Start();
        }
        void init_systray()
        {
            cb_systray.CheckedChanged += systray_click;
            ntfico.DoubleClick += systray_double_click;
            ntfico.Icon = Icon;
            Resize += form_resize;
            if (rconf.get("systray") == "true") cb_systray.Checked = true;
        }
        void save_systray()
        {
            if (onloading) return;
            rconf.set("systray", cb_systray.Checked ? "true" : "false");
        }
        void form_resize(object sender, EventArgs e)
        {
            if (WindowState == FormWindowState.Minimized && ntfico.Visible) Visible = false;
        }
        void systray_click(object sender, EventArgs e)
        {
            ntfico.Visible = ((CheckBox)sender).Checked;
            ShowInTaskbar = !ntfico.Visible;
            save_systray();
        }
        void systray_double_click(object sender, EventArgs e)
        {
            Visible = (WindowState == FormWindowState.Minimized);
            WindowState = (WindowState == FormWindowState.Minimized) ? FormWindowState.Normal : FormWindowState.Minimized;
            if (WindowState == FormWindowState.Normal) Activate();
        }
        void top_click(object sender, EventArgs e)
        {
            TopMost = ((CheckBox)sender).Checked;
        }
        void autostart_click(object sender, EventArgs e)
        {
            CheckBox cb = (CheckBox)sender;
            if (cb.Checked) sh.reg_autostart();
            else sh.dereg_autostart();
        }
    }
}
