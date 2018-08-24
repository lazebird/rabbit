using lazebird.rabbit.common;
using lazebird.vgen.ver;
using System;
using System.Collections.Generic;
using System.Configuration;
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
        // Environment.ExpandEnvironmentVariables(@"%userprofile%\appdata\local");
        string profileuri = Path.GetDirectoryName(ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.PerUserRoamingAndLocal).FilePath);
        string helpurl = "https://code.aliyun.com/lazebird/rabbit/blob/master/doc/manual.md";
        string versionuri = "https://code.aliyun.com/lazebird/rabbit/raw/master/release/version.txt";
        string binaryuri = "https://code.aliyun.com/lazebird/rabbit/raw/master/release/sRabbit.exe";
        void init_form_setting()
        {
            setlog = new rlog(setting_output);
            pver = Assembly.GetExecutingAssembly().GetName().Version;
            List<string> list = new List<string>();
            list.Add("System");
            list.Add("English");
            list.Add("中文");
            lang_cb.DataSource = list;
            lang_cb.Text = Language.Getlang();
            setlog.write("Language: " + Language.Getlang());
            lang_cb.SelectedIndexChanged += lang_opt_SelectedIndexChanged;
            cb_systray.CheckedChanged += systray_click;
            ntfico.DoubleClick += systray_double_click;
            ntfico.Icon = Icon;
            Resize += form_resize;
            cb_top.CheckedChanged += top_click;
            cb_autostart.CheckedChanged += autostart_click;
            cb_autoupdate.CheckedChanged += autoupdate_click;
            link_prj.LinkClicked += url_click;
            link_prof.LinkClicked += url_click;
            link_help.LinkClicked += url_click;
            link_ver.Text = pver.ToString() + " (" + appver.v.ToString() + ")";
            link_ver.LinkClicked += ver_click;
        }
        void lang_opt_SelectedIndexChanged(object sender, EventArgs e)
        {
            string lang = lang_cb.SelectedItem as string;
            if (lang_cb.Text == "System") rconf.set("lang", "");
            else rconf.set("lang", lang_cb.Text);
            setlog.write("Set Language: " + lang_cb.Text);
            setlog.write("Restart App to take effect!");
        }
        void url_click(object sender, LinkLabelLinkClickedEventArgs e)
        {
            if (sender == link_prj)
                System.Diagnostics.Process.Start(prjurl);
            else if (sender == link_prof)
                System.Diagnostics.Process.Start(profileuri);
            else if (sender == link_help)
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
                string newbinpath = Application.ExecutablePath + ".new";
                MessageBoxButtons mbox = MessageBoxButtons.YesNo;
                DialogResult res = MessageBox.Show("New Version!!!\r\n" + v.ToString() + "\r\nDownload?", "Rabbit Upgrade", mbox);
                if (res == DialogResult.Yes)
                {
                    download2file(binaryuri, newbinpath);
                    DialogResult res2 = MessageBox.Show("Restart & Upgrade?", "Rabbit Upgrade", mbox);
                    if (res2 == DialogResult.Yes)
                    {
                        upgrade upg = new upgrade(Application.ExecutablePath);
                        upg.run();
                        Application.Exit();
                    }
                    else MessageBox.Show("New version saved to " + newbinpath, "Rabbit Upgrade");

                }
                else setlog.write("New version: " + v.ToString() + ", click the homepage to download it.");

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
        void form_resize(object sender, EventArgs e)
        {
            if (WindowState == FormWindowState.Minimized && ntfico.Visible) Visible = false;
        }
        void systray_click(object sender, EventArgs e)
        {
            ntfico.Visible = ((CheckBox)sender).Checked;
            ShowInTaskbar = !ntfico.Visible;
            setting_saveconf();
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
        void autoupdate_click(object sender, EventArgs e)
        {
            setting_saveconf();
        }
        void setting_readconf()
        {
            if (File.Exists(upgrade.scriptpath)) File.Delete(upgrade.scriptpath);
            if (sh.autostart_exist()) cb_autostart.Checked = true;
            if (rconf.get("systray") == "true") cb_systray.Checked = true;
            if (rconf.get("autoupdate") == "true") cb_autoupdate.Checked = true;
            if (cb_autoupdate.Checked) ver_click(null, null);
        }
        void setting_saveconf()
        {
            if (onloading) return;
            rconf.set("systray", cb_systray.Checked ? "true" : "false");
            rconf.set("autoupdate", cb_autoupdate.Checked ? "true" : "false");
        }
    }
}
