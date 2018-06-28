using lazebird.rabbit.common;
using lazebird.rabbit.http;
using lazebird.rabbit.ping;
using lazebird.rabbit.tftp;
using lazebird.rabbit.ftp;
using lazebird.rabbit.dhcp;
using System;
using System.Collections;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        Hashtable texthash;
        Hashtable btnhash;
        Hashtable formhash;
        Hashtable indexhash;
        myping ping;
        httpd httpd;
        tftpd tftpd;
        ftpd ftp;
        dhcpd dhcp;
        mylog pinglog;
        mylog httpdlog;
        mylog tftpdlog;
        public Form1()
        {
            InitializeComponent();
            AcceptButton = btn_ping;
            //CancelButton = Application.Exit(0);
            CheckForIllegalCrossThreadCalls = false;
            pinglog = new mylog(ping_output);
            httpdlog = new mylog(httpd_output);
            tftpdlog = new mylog(tftpd_output);
            ping = new myping(pinglog, ping_stop_cb);
            httpd = new httpd(httpdlog);
            httpd.init_mime(myconf.get("mime"));
            tftpd = new tftpd(tftpdlog);
            btn_ping.Click += new EventHandler(ping_click);
            btn_ping_log.Click += new EventHandler(ping_log_click);
            btn_httpd.Click += new EventHandler(httpd_click);
            btn_http_dir.Click += new EventHandler(httpd_dir_click);
            tftp_dirbtn1.Click += new EventHandler(tftpd_dir1_click);
            tftp_dirbtn2.Click += new EventHandler(tftpd_dir2_click);
            tftp_dirbtn3.Click += new EventHandler(tftpd_dir3_click);
            tftp_dirbtn4.Click += new EventHandler(tftpd_dir4_click);
            tftp_dirbtn5.Click += new EventHandler(tftpd_dir5_click);
            init_elements();
        }
        bool onloading = false;
        protected override void OnLoad(EventArgs e)
        {
            onloading = true;
            base.OnLoad(e);
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
            texthash.Add("tftp_dir1", tftp_dirtext1);
            texthash.Add("tftp_dir2", tftp_dirtext2);
            texthash.Add("tftp_dir3", tftp_dirtext3);
            texthash.Add("tftp_dir4", tftp_dirtext4);
            texthash.Add("tftp_dir5", tftp_dirtext5);
            btnhash.Add("ping_btn", btn_ping);
            btnhash.Add("httpd_btn", btn_httpd);
            formhash.Add("form", this);
            indexhash.Add("tabs", tabs);
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
                if (myconf.get(key) == "停止")
                {
                    ((TabControl)indexhash["tabs"]).SelectedIndex = (int)indexhash[key];
                    ((Button)btnhash[key]).PerformClick();
                }
            }
            ((TabControl)indexhash["tabs"]).SelectedIndex = int.Parse(myconf.get("tabs"));
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
            myconf.set("tabs", ((TabControl)indexhash["tabs"]).SelectedIndex.ToString());
            foreach (string key in btnhash.Keys)
            {
                myconf.set(key, ((Button)btnhash[key]).Text);
            }
        }
        private void ping_click(object sender, EventArgs evt)
        {
            if (((Button)btnhash["ping_btn"]).Text == "开始")
            {
                try
                {
                    ((Form)formhash["form"]).Text = ((TextBox)texthash["ping_addr"]).Text;
                    ((Button)btnhash["ping_btn"]).Text = "停止";
                    pinglog.setfile(((TextBox)texthash["ping_logfile"]).Text);
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
            SaveFileDialog fd = new SaveFileDialog();
            //filename.InitialDirectory = Application.StartupPath;
            fd.Filter = "文本文件 (*.txt)|*.txt|All files (*.*)|*.*";
            fd.RestoreDirectory = true;
            fd.CreatePrompt = true;
            fd.OverwritePrompt = true;
            fd.InitialDirectory = Environment.CurrentDirectory;
            if (fd.ShowDialog() == DialogResult.OK)
            {
                ((TextBox)texthash["ping_logfile"]).Text = fd.FileName;
            }
        }
        private void httpd_click(object sender, EventArgs evt)
        {
            if (((Button)btnhash["httpd_btn"]).Text == "开始")
            {
                ((Button)btnhash["httpd_btn"]).Text = "停止";
                httpd.set_dir(((TextBox)texthash["http_dir"]).Text);
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
            dialog.SelectedPath = Environment.CurrentDirectory;
            dialog.Description = "请选择文件路径";
            if (dialog.ShowDialog() == DialogResult.OK)
            {
                text_http_dir.Text = dialog.SelectedPath;
            }
        }
        private void tftpd_dir_set(TextBox t)
        {
            FolderBrowserDialog dialog = new FolderBrowserDialog();
            dialog.SelectedPath = t.Text;
            dialog.Description = "请选择文件路径";
            if (dialog.ShowDialog() == DialogResult.OK)
            {
                tftpd.del_dir(t.Text);
                t.Text = dialog.SelectedPath;
                tftpd.add_dir(t.Text);
                saveconf();
            }
        }
        private void tftpd_dir1_click(object sender, EventArgs e)
        {
            tftpd_dir_set(tftp_dirtext1);
        }
        private void tftpd_dir2_click(object sender, EventArgs e)
        {
            tftpd_dir_set(tftp_dirtext2);
        }
        private void tftpd_dir3_click(object sender, EventArgs e)
        {
            tftpd_dir_set(tftp_dirtext3);
        }
        private void tftpd_dir4_click(object sender, EventArgs e)
        {
            tftpd_dir_set(tftp_dirtext4);
        }
        private void tftpd_dir5_click(object sender, EventArgs e)
        {
            tftpd_dir_set(tftp_dirtext5);
        }
    }
}
