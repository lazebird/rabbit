using System;
using System.Collections;
using System.Windows.Forms;
using System.Threading;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        Hashtable texthash;
        Hashtable btnhash;
        Hashtable formhash;
        Hashtable indexhash;
        public Form1()
        {
            InitializeComponent();
            CheckForIllegalCrossThreadCalls = false;
            init_form_ping();
            init_form_scan();
            init_form_http();
            init_form_tftp();
            init_form_setting();
            init_hash();
        }
        bool onloading = false;
        protected override void OnLoad(EventArgs e)
        {
            onloading = true;
            base.OnLoad(e);
            Thread th = new Thread(bar_test);
            th.IsBackground = true;
            th.Start();
            readconf();
            onloading = false;
        }
        protected override bool ProcessDialogKey(Keys keyData)
        {
            if (keyData == Keys.Escape)
            {
                this.Close();
                return true;
            }
            else if (keyData == Keys.Enter)
            {
                foreach (DictionaryEntry de in indexhash)
                {
                    if ((int)de.Value == tabs.SelectedIndex)
                        if (btnhash.ContainsKey(de.Key))
                            ((Button)btnhash[de.Key]).PerformClick();
                }
            }
            return base.ProcessDialogKey(keyData);
        }
        void init_hash()
        {
            texthash = new Hashtable();
            btnhash = new Hashtable();
            formhash = new Hashtable();
            indexhash = new Hashtable();
            texthash.Add("ping_addr", text_addr);
            texthash.Add("ping_timeout", text_interval);
            texthash.Add("ping_times", text_count);
            texthash.Add("ping_logfile", text_logpath);
            texthash.Add("http_port", text_http_port);
            texthash.Add("http_dir", text_http_dir);
            texthash.Add("tftp_timeout", text_tftptout);
            texthash.Add("tftp_retry", text_tftpretry);
            texthash.Add("tftp_dir1", tftp_dirtext1);
            texthash.Add("tftp_dir2", tftp_dirtext2);
            texthash.Add("tftp_dir3", tftp_dirtext3);
            texthash.Add("scan_ipstart", text_scanstart);
            texthash.Add("scan_ipend", text_scanend);
            btnhash.Add("ping_btn", btn_ping);
            indexhash.Add("ping_btn", 0);
            btnhash.Add("httpd_btn", btn_httpd);
            indexhash.Add("httpd_btn", 2);
            btnhash.Add("tftpd_btn", tftpd_btn);
            indexhash.Add("tftpd_btn", 3);
            formhash.Add("form", this);
        }
        void conf_log(string msg)
        {
            //tftpdlog.write(msg);
        }
        void readconf()
        {
            foreach (string key in texthash.Keys)
            {
                conf_log("G: " + key + " - " + rconf.get(key));
                ((TextBox)texthash[key]).Text = rconf.get(key);
            }
            foreach (string key in btnhash.Keys)
            {
                conf_log("G: " + key + " - " + rconf.get(key));
                if (rconf.get(key) != "" && Language.trans(rconf.get(key)) != ((Button)btnhash[key]).Text)
                {
                    tabs.SelectedIndex = (int)indexhash[key];
                    ((Button)btnhash[key]).PerformClick();
                }
            }
            conf_log("G: " + "tabs" + " - " + rconf.get("tabs"));
            tabs.SelectedIndex = int.Parse(rconf.get("tabs"));
        }
        void saveconf()
        {
            if (onloading)
            {
                return;
            }
            foreach (string key in texthash.Keys)
            {
                rconf.set(key, ((TextBox)texthash[key]).Text);
                conf_log("S: " + key + " - " + ((TextBox)texthash[key]).Text);
            }
            rconf.set("tabs", tabs.SelectedIndex.ToString());
            conf_log("S: " + "tabs" + " - " + tabs.SelectedIndex.ToString());
            foreach (string key in btnhash.Keys)
            {
                rconf.set(key, ((Button)btnhash[key]).Text);
                conf_log("S: " + key + " - " + ((Button)btnhash[key]).Text);
            }
        }
    }
}
