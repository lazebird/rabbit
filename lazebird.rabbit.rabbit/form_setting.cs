using lazebird.rabbit.common;
using System;
using System.Collections.Generic;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rlog setlog;
        void init_form_setting()
        {
            setlog = new rlog(setting_output);
            prjurl.LinkClicked += url_click;
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
    }
}
