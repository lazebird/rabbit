using lazebird.rabbit.tftp;
using lazebird.rabbit.ftp;
using lazebird.rabbit.dhcp;
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
            init_form_http();
            init_form_tftp();
            init_hash();
        }
        bool onloading = false;
        protected override void OnLoad(EventArgs e)
        {
            onloading = true;
            base.OnLoad(e);
            new Thread(bar_test).Start();
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
        private void init_hash()
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
            texthash.Add("tftp_dir1", tftp_dirtext1);
            texthash.Add("tftp_dir2", tftp_dirtext2);
            texthash.Add("tftp_dir3", tftp_dirtext3);
            texthash.Add("tftp_dir4", tftp_dirtext4);
            texthash.Add("tftp_dir5", tftp_dirtext5);
            btnhash.Add("ping_btn", btn_ping);
            btnhash.Add("httpd_btn", btn_httpd);
            btnhash.Add("tftpd_btn", tftpd_btn);
            formhash.Add("form", this);
            indexhash.Add("ping_btn", 0);
            indexhash.Add("httpd_btn", 1);
        }
        private void readconf()
        {
            foreach (string key in texthash.Keys)
            {
                ((TextBox)texthash[key]).Text = myconf.get(key);
                if (key.Contains("tftp_dir"))
                {
                    tftpd.add_dir(((TextBox)texthash[key]).Text);
                }
            }
            foreach (string key in btnhash.Keys)
            {
                if (myconf.get(key) != ((Button)btnhash[key]).Text)
                {
                    tabs.SelectedIndex = (int)indexhash[key];
                    ((Button)btnhash[key]).PerformClick();
                }
            }
            tabs.SelectedIndex = int.Parse(myconf.get("tabs"));
        }
        private void saveconf()
        {
            if (onloading)
            {
                return;
            }
            foreach (string key in texthash.Keys)
            {
                myconf.set(key, ((TextBox)texthash[key]).Text);
            }
            myconf.set("tabs", tabs.SelectedIndex.ToString());
            foreach (string key in btnhash.Keys)
            {
                myconf.set(key, ((Button)btnhash[key]).Text);
            }
        }
    }
}
