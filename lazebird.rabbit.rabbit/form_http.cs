using lazebird.rabbit.common;
using lazebird.rabbit.http;
using lazebird.rabbit.shell;
using System;
using System.Collections;
using System.Drawing;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rhttpd httpd;
        rlog httpdlog;
        rshell sh;
        Hashtable httpd_phash;
        void init_form_http()
        {
            sh = new rshell("Rabbit", Application.ExecutablePath, "Add to Rabbit.http");
            httpd_phash = new Hashtable();
            //httpd_output.HorizontalScrollbar = true;
            //httpd_output.HorizontalExtent = 5000;
            httpdlog = new rlog(httpd_output);
            httpd = new rhttpd(httpd_log_func);
            httpd.init_mime(rconf.get("mime"));
            btn_httpd.Click += new EventHandler(httpd_click);
            if (sh.file_exist()) cb_http_shell.Checked = true;
            cb_http_shell.CheckedChanged += new EventHandler(http_shell_click);
            cb_http_index.CheckedChanged += new EventHandler(http_index_click);
            fp_httpd.AutoScroll = true;
            init_comsrv();
        }
        void httpd_log_func(string msg)
        {
            httpdlog.write(msg);
        }
        void httpd_click(object sender, EventArgs evt)
        {
            if (((Button)btnhash["httpd_btn"]).Text == Language.trans("开始"))
            {
                ((Button)btnhash["httpd_btn"]).Text = Language.trans("停止");
                httpd.start(int.Parse(((TextBox)texthash["http_port"]).Text));
            }
            else
            {
                ((Button)btnhash["httpd_btn"]).Text = Language.trans("开始");
                httpd.stop();
            }
            saveconf(); // save empty config to restore default config
        }
        void init_comsrv()
        {
            Thread t = new Thread(com_task);
            t.IsBackground = true;
            t.Start();
        }
        void com_task()
        {
            int port = 2222; // internal com udp port
            try
            {
                IPEndPoint r = new IPEndPoint(IPAddress.Parse("127.0.0.1"), port);
                UdpClient uc = new UdpClient(r);
                byte[] rcvBuffer;
                while (true)
                {
                    rcvBuffer = uc.Receive(ref r);
                    httpd_add_path(Encoding.Default.GetString(rcvBuffer));
                }
            }
            catch (Exception e)
            {
                httpd_log_func("!E: " + e.ToString());
            }
        }
        void http_shell_click(object sender, EventArgs e)
        {
            CheckBox cb = (CheckBox)sender;
            //httpd_log_func("I: current checked " + cb.Checked + " exist " + sh.file_exist());
            if (cb.Checked) { sh.reg_file(); sh.reg_dir(); }
            else { sh.dereg_file(); sh.dereg_dir(); }
        }
        void http_index_click(object sender, EventArgs e)
        {
            CheckBox cb = (CheckBox)sender;
            httpd.set_auto_index(cb.Checked);
            saveconf();
        }
        void httpd_dir_click(object sender, EventArgs evt)
        {
            TextBox tb = (TextBox)sender;
            string p = tb.Text;
            if (File.Exists(p)) httpd.del_file(p);
            else if (Directory.Exists(p)) httpd.del_dir(p);
            httpd_phash.Remove(tb);
            fp_httpd.Controls.Remove(tb);
            saveconf();
        }
        void httpd_add_path(string p)
        {
            p = Path.GetFullPath(p);
            if (httpd_phash.ContainsValue(p)) return;
            if (this.InvokeRequired)
            {
                this.Invoke(new MethodInvoker(delegate { httpd_add_path(p); }));
                return;
            }
            TextBox tb = new TextBox();
            tb.ReadOnly = true;
            tb.BackColor = Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            tb.BorderStyle = BorderStyle.None;
            tb.ForeColor = Color.White;
            tb.Text = p;
            tb.Width = 640;
            //tb.Anchor = (AnchorStyles.Top | AnchorStyles.Left | AnchorStyles.Right);
            tb.DoubleClick += new EventHandler(httpd_dir_click);
            fp_httpd.Controls.Add(tb);
            httpd_phash.Add(tb, p);
            if (File.Exists(p)) httpd.add_file(p);
            else if (Directory.Exists(p)) httpd.add_dir(p);
            saveconf();
        }
        void httpd_readconf()
        {
            string[] paths = rconf.get("http_dirs").Split(';');
            foreach (string path in paths) if (path != "") httpd_add_path(path);
            if (rconf.get("http_auto_index") != "")
            {
                cb_http_index.Checked = true;
                httpd.set_auto_index(cb_http_index.Checked);
            }
        }
        void httpd_saveconf()
        {
            if (onloading) return;
            string dirs = "";
            foreach (string path in httpd_phash.Values)
            {
                dirs += path + ";";
            }
            rconf.set("http_dirs", dirs);
            if (cb_http_index.Checked) rconf.set("http_auto_index", "true");
        }
    }
}
