using lazebird.rabbit.common;
using System;
using System.Collections.Generic;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        mylog setlog;
        void init_form_setting()
        {
            setlog = new mylog(setting_output);
            List<string> list = new List<string>();
            list.Add("System");
            list.Add("English");
            list.Add("中文");
            lang_opt.DataSource = list;
            lang_opt.Text = Language.Getlang();
            lang_opt.SelectedIndexChanged += lang_opt_SelectedIndexChanged;
            setlog.write("Language: " + Language.Getlang());
        }
        void lang_opt_SelectedIndexChanged(object sender, EventArgs e)
        {
            // 将被选中的项目强制转换为MyItem
            string lang = lang_opt.SelectedItem as string;
            if (lang_opt.Text == "System")
            {
                rconf.set("lang", "");
            }
            else
            {
                rconf.set("lang", lang_opt.Text);
            }
            setlog.write("Set Language: " + lang_opt.Text);
            setlog.write("Restart App to take effect!");
        }
    }
}
