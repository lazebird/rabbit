using lazebird.rabbit.common;
using lazebird.rabbit.fs;
using lazebird.rabbit.http;
using System;
using System.Collections;
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
        rpanel http_fpannel;
        rhttpd httpd;
        rlog httpdlog;
        rshell sh;
        Hashtable httpd_phash;
        int httpport;
        Hashtable http_opts;
        void init_form_http()
        {
            http_fpannel = new rpanel(fp_httpd_file);
            sh = new rshell("Rabbit", Application.ExecutablePath, "Add to Rabbit.http");
            httpd_phash = new Hashtable();
            //httpd_output.HorizontalScrollbar = true;
            //httpd_output.HorizontalExtent = 5000;
            httpdlog = new rlog(lb_httpd);
            httpd = new rhttpd(httpd_log_func);
            if (sh.file_exist()) cb_http_shell.Checked = true;
            cb_http_shell.CheckedChanged += new EventHandler(http_shell_click);
            init_comsrv();
        }
        void httpd_log_func(string msg)
        {
            httpdlog.write(msg);
        }
        void httpd_click(object sender, EventArgs evt)
        {
            try
            {
                http_parse_args();
                if (((Button)btnhash["http_btn"]).Text == Language.trans("开始"))
                {
                    ((Button)btnhash["http_btn"]).Text = Language.trans("停止");
                    httpd.start(httpport, http_opts);
                }
                else
                {
                    ((Button)btnhash["http_btn"]).Text = Language.trans("开始");
                    httpd.stop();
                }
            }
            catch (Exception e)
            {
                httpd_log_func("!E: " + e.ToString());
            }
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
        void httpd_dir_click(object sender, EventArgs evt)
        {
            TextBox tb = (TextBox)sender;
            string p = tb.Text;
            if (File.Exists(p)) httpd.del_file(p);
            else if (Directory.Exists(p)) httpd.del_dir(p);
            httpd_phash.Remove(tb);
            http_fpannel.del(tb);
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
            TextBox tb = http_fpannel.add(p, null, httpd_dir_click);
            httpd_phash.Add(tb, p);
            if (File.Exists(p)) httpd.add_file(p);
            else if (Directory.Exists(p)) httpd.add_dir(p);
        }
        void http_parse_args()
        {
            int.TryParse(((TextBox)texthash["http_port"]).Text, out httpport);
            http_opts = ropt.parse_opts(text_httpopt.Text);
        }
        void httpd_conf_set(string name, string val)
        {
            string[] paths = val.Split(';');
            foreach (string path in paths) if (path != "") httpd_add_path(path);
        }
        string httpd_conf_get(string name)
        {
            string dirs = "";
            foreach (string path in httpd_phash.Values) dirs += path + ";";
            return dirs;
        }
    }
}
