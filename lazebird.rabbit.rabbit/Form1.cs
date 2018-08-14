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
            init_form_tftpd();
            init_form_tftpc();
            init_form_plan();
            init_form_chat();
            init_form_setting();
            init_hash();
        }
        bool onloading = false;
        protected override void OnLoad(EventArgs e)
        {
            base.OnLoad(e);
            onloading = true;
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
            texthash.Add("ping_addr", text_pingaddr);
            texthash.Add("ping_opt", text_pingopt);
            texthash.Add("http_port", text_http_port);
            texthash.Add("tftpd_opt", text_tftpdopt);
            texthash.Add("scan_ipstart", text_scanstart);
            texthash.Add("scan_ipend", text_scanend);
            texthash.Add("scan_opt", text_scanopt);
            texthash.Add("tftpc_addr", text_tftpcaddr);
            texthash.Add("tftpc_opt", text_tftpcopt);
            btnhash.Add("ping_btn", btn_ping);
            indexhash.Add("ping_btn", 0);
            btnhash.Add("http_btn", btn_httpd);
            indexhash.Add("http_btn", 2);
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
            conf_log("G: " + "tabindex" + " - " + rconf.get("tabindex"));
            tabs.SelectedIndex = int.Parse(rconf.get("tabindex"));
            httpd_readconf();
            tftpd_readconf();
            plan_readconf();
            chat_readconf();
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
            rconf.set("tabindex", tabs.SelectedIndex.ToString());
            conf_log("S: " + "tabindex" + " - " + tabs.SelectedIndex.ToString());
            foreach (string key in btnhash.Keys)
            {
                rconf.set(key, ((Button)btnhash[key]).Text);
                conf_log("S: " + key + " - " + ((Button)btnhash[key]).Text);
            }
            httpd_saveconf();
            tftpd_saveconf();
            plan_saveconf();
            chat_saveconf();
        }
        Hashtable parse_opts(string s)
        {
            Hashtable h = new Hashtable();
            string[] attrs = s.Split(';');
            foreach (string attr in attrs)
            {
                if (attr.Length <= 0) continue;
                string[] opts = attr.Split('=');
                if (opts.Length != 2) continue;
                if (h.ContainsKey(opts[0])) h.Remove(opts[0]);
                h.Add(opts[0], opts[1]);
            }
            return h;
        }
    }
}
