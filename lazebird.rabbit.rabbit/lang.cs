using System;
using System.Collections;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public class Language
    {

        /// <summary>
        /// 设置当前程序的界面语言
        /// </summary>
        /// <param name="lang">language:zh-CN, en-US</param>
        /// <param name="form">窗体实例</param>
        /// <param name="formType">窗体类型</param>
        public static void SetLang(string lang, Form form, Type formType)
        {
            System.Threading.Thread.CurrentThread.CurrentUICulture = new System.Globalization.CultureInfo(lang);
            if (form != null)
            {
                System.ComponentModel.ComponentResourceManager resources = new System.ComponentModel.ComponentResourceManager(formType);
                resources.ApplyResources(form, "$this");
                AppLang(form, resources);
            }
        }
        static string language;
        public static string Getsetting()
        {
            language = "";
            if (language != "中文" && language != "English")
            {
                //language = (System.Threading.Thread.CurrentThread.CurrentUICulture.Name == "zh-CN") ? "中文" : "English";
                language = "English";   // default use english
            }
            return (language == "中文") ? "zh-CN" : "";
        }
        public static string Getlang()
        {
            Getsetting();
            return language;
        }
        public static string trans(string key)
        {
            Hashtable zhhash = new Hashtable();
            Hashtable enhash = new Hashtable();
            zhhash.Add("Start", "开始");
            zhhash.Add("Stop", "停止");
            enhash.Add("开始", "Start");
            enhash.Add("停止", "Stop");
            Getsetting();
            if (false && language == "中文")
            {
                return zhhash.ContainsKey(key) ? (string)zhhash[key] : key;
            }
            else
            {
                return enhash.ContainsKey(key) ? (string)enhash[key] : key;
            }
        }

        /// <summary>
        /// 遍历窗体所有控件，针对其设置当前界面语言
        /// </summary>
        /// <param name="control"></param>
        /// <param name="resources"></param>
        static void AppLang(Control control, System.ComponentModel.ComponentResourceManager resources)
        {
            if (control is MenuStrip)
            {
                resources.ApplyResources(control, control.Name);
                MenuStrip ms = (MenuStrip)control;
                if (ms.Items.Count > 0)
                {
                    foreach (ToolStripMenuItem c in ms.Items)
                    {
                        AppLang(c, resources);
                    }
                }
            }

            foreach (Control c in control.Controls)
            {
                resources.ApplyResources(c, c.Name);
                AppLang(c, resources);
            }
        }
        /// <summary>
        /// 遍历菜单
        /// </summary>
        /// <param name="item"></param>
        /// <param name="resources"></param>
        static void AppLang(ToolStripMenuItem item, System.ComponentModel.ComponentResourceManager resources)
        {
            if (item is ToolStripMenuItem)
            {
                resources.ApplyResources(item, item.Name);
                ToolStripMenuItem tsmi = (ToolStripMenuItem)item;
                if (tsmi.DropDownItems.Count > 0)
                {
                    for (int i = 0; i < tsmi.DropDownItems.Count; i++)
                    {
                        if (tsmi.DropDownItems[i] is ToolStripMenuItem)
                        {
                            {
                                AppLang((ToolStripMenuItem)tsmi.DropDownItems[i], resources);

                            }
                        }
                    }
                }
            }
        }
    }
}
