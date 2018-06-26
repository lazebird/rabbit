using rabbit.common;
using rabbit.http;
using rabbit.ping;
using rabbit.tftp;
using rabbit.ftp;
using rabbit.dhcp;
using System;
using System.Collections;
using System.Windows.Forms;
using System.IO;

namespace rabbit
{
    public partial class Form1 : Form
    {
        myping ping;
        httpd httpd;
        tftpd tftp;
        ftpd ftp;
        dhcpd dhcp;
        mylog pinglog;
        mylog httpdlog;
        Hashtable texthash;
        Hashtable btnhash;
        Hashtable formhash;
        Hashtable indexhash;
        public Form1()
        {
            InitializeComponent();
            AcceptButton = btn_ping;
            //CancelButton = Application.Exit(0);
            CheckForIllegalCrossThreadCalls = false;
            pinglog = new mylog(ping_output);
            httpdlog = new mylog(httpd_output);
            ping = new myping(pinglog, ping_stop_cb);
            httpd = new httpd(httpdlog);
            init_elements();
        }
        protected override void OnLoad(EventArgs e)
        {
            base.OnLoad(e);
            readconf();
        }
        protected override bool ProcessDialogKey(Keys keyData)
        {
            if (keyData == Keys.Escape)
            {
                this.Close();
                return true;
            }
            return base.ProcessDialogKey(keyData);
        }
        public void ping_stop_cb()
        {
            ((Form)formhash["form"]).Text = "Rabbit";
            ((Button)btnhash["ping_btn"]).Text = "开始";
        }
        private void init_elements()
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
            btnhash.Add("ping_btn", btn_ping);
            btnhash.Add("httpd_btn", btn_httpd);
            formhash.Add("form", this);
            indexhash.Add("tabs", tabs);
            btn_ping.Click += new EventHandler(ping_click);
            btn_ping_log.Click += new EventHandler(ping_log_click);
            btn_httpd.Click += new EventHandler(httpd_click);
            btn_http_dir.Click += new EventHandler(httpd_dir_click);
        }
        private void readconf()
        {
            foreach (string key in texthash.Keys)
            {
                ((TextBox)texthash[key]).Text = myconf.get(key);
            }
            ((TabControl)indexhash["tabs"]).SelectedIndex = int.Parse(myconf.get("tabs"));
            foreach (string key in btnhash.Keys)
            {
                if (myconf.get(key) == "停止")
                {
                    ((Button)btnhash[key]).PerformClick();
                }
            }
        }
        private void saveconf()
        {
            foreach (string key in texthash.Keys)
            {
                myconf.set(key, ((TextBox)texthash[key]).Text);
            }
            myconf.set("tabs", ((TabControl)indexhash["tabs"]).SelectedIndex.ToString());
            foreach (string key in btnhash.Keys)
            {
                myconf.set(key, ((Button)btnhash[key]).Text);
            }
        }
        private void ping_click(object sender, EventArgs evt)
        {
            pinglog.setfile(((TextBox)texthash["ping_logfile"]).Text);
            if (((Button)btnhash["ping_btn"]).Text == "开始")
            {
                try
                {
                    ((Form)formhash["form"]).Text = ((TextBox)texthash["ping_addr"]).Text;
                    ((Button)btnhash["ping_btn"]).Text = "停止";
                    pinglog.setfile(text_logpath.Text);
                    pinglog.clear();
                    ping.start(((TextBox)texthash["ping_addr"]).Text, int.Parse(((TextBox)texthash["ping_timeout"]).Text), int.Parse(((TextBox)texthash["ping_times"]).Text));
                }
                catch (Exception e)
                {
                    pinglog.write("Error: " + e.Message);
                }
            }
            else
            {
                ping.stop();
            }
            saveconf(); // save empty config to restore default config
        }
        private void ping_log_click(object sender, EventArgs e)
        {
            SaveFileDialog filename = new SaveFileDialog();
            //filename.InitialDirectory = Application.StartupPath;
            filename.Filter = "文本文件 (*.txt)|*.txt|All files (*.*)|*.*";
            filename.RestoreDirectory = true;
            filename.CreatePrompt = true;
            filename.OverwritePrompt = true;
            if (filename.ShowDialog() == DialogResult.OK)
            {
                ((TextBox)texthash["ping_logfile"]).Text = filename.FileName;
            }
        }
        private void httpd_click(object sender, EventArgs evt)
        {
            httpd.set_dir(((TextBox)texthash["http_dir"]).Text);
            if (((Button)btnhash["httpd_btn"]).Text == "开始")
            {
                ((Button)btnhash["httpd_btn"]).Text = "停止";
                httpd.start(int.Parse(((TextBox)texthash["http_port"]).Text));
            }
            else
            {
                ((Button)btnhash["httpd_btn"]).Text = "开始";
                httpd.stop();
            }
            saveconf(); // save empty config to restore default config
        }
        private void httpd_dir_click(object sender, EventArgs e)
        {
            FolderBrowserDialog dialog = new FolderBrowserDialog();
            dialog.Description = "请选择文件路径";
            if (dialog.ShowDialog() == DialogResult.OK)
            {
                text_http_dir.Text = dialog.SelectedPath;
            }
        }
    }
}
