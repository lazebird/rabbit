﻿using lazebird.rabbit.common;
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
        void init_form_setting()
        {
            setlog = new rlog(setting_output);
            pver = Assembly.GetExecutingAssembly().GetName().Version;
            link_ver.Text = pver.ToString();
            link_ver.LinkClicked += ver_click;
            link_prj.LinkClicked += url_click;
            link_prof.Text = @"%userprofile%\appdata\local";
            link_prof.LinkClicked += path_click;
            List<string> list = new List<string>();
            list.Add("System");
            list.Add("English");
            list.Add("中文");
            lang_cb.DataSource = list;
            lang_cb.Text = Language.Getlang();
            lang_cb.SelectedIndexChanged += lang_opt_SelectedIndexChanged;
            setlog.write("Language: " + Language.Getlang());
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
        void url_click(object sender, LinkLabelLinkClickedEventArgs e)
        {
            System.Diagnostics.Process.Start(((LinkLabel)sender).Text);
        }
        void path_click(object sender, LinkLabelLinkClickedEventArgs e)
        {
            string path = Environment.ExpandEnvironmentVariables(((LinkLabel)sender).Text);
            System.Diagnostics.Process.Start(path);
        }
        string getbyuri(string uri, string prefix)
        {
            WebClient wc = new WebClient();
            string data = wc.DownloadString(uri);
            wc.Dispose();
            int start = data.IndexOf(prefix);
            if (start < 0) return "";
            data = data.Substring(start + prefix.Length);
            int end = data.IndexOf("#");
            return (end < 0) ? data : data.Substring(0, end);
        }
        void ver_check()
        {
            try
            {
                string s = getbyuri("https://raw.githubusercontent.com/lazebird/rabbit/master/doc/releasenotes.md", "## Latest Version: ");
                Version newver = new Version(s);
                if (pver.CompareTo(newver) < 0)
                    setlog.write("The newest version is " + s + ", click the homepage to download it.");
                else
                    setlog.write("Version is up to date!");
            }
            catch (Exception e)
            {
                setlog.write("!E: " + e.ToString());
            }
        }
        void ver_click(object sender, LinkLabelLinkClickedEventArgs evt)
        {
            Thread t = new Thread(ver_check);
            t.IsBackground = true;
            t.Start();
        }
    }
}
