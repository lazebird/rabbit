using lazebird.rabbit.common;
using lazebird.rabbit.http;
using Microsoft.Win32;
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
        Hashtable httpd_phash;
        void init_form_http()
        {
            httpd_phash = new Hashtable();
            //httpd_output.HorizontalScrollbar = true;
            //httpd_output.HorizontalExtent = 5000;
            httpdlog = new rlog(httpd_output);
            httpd = new rhttpd(httpd_log_func);
            httpd.init_mime(rconf.get("mime"));
            btn_httpd.Click += new EventHandler(httpd_click);
            btn_http_reg.Click += new EventHandler(http_reg_shell);
            btn_http_dereg.Click += new EventHandler(http_dereg_shell);
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
        string shellname = "Rabbit";
        string menuname = "Add to Rabbit.http";
        void reg_shell(RegistryKey shell)
        {
            RegistryKey custom = shell.CreateSubKey(shellname);
            custom.SetValue("", menuname);
            RegistryKey cmd = custom.CreateSubKey("command");
            cmd.SetValue("", Application.ExecutablePath + " %1");
            cmd.Close();
            custom.Close();
        }
        void dereg_shell(RegistryKey shell)
        {
            shell.DeleteSubKeyTree(shellname);
        }
        void http_reg_shell(object sender, EventArgs e)
        {
            RegistryKey shell = Registry.ClassesRoot.OpenSubKey(@"*\shell", true);
            reg_shell(shell);
            shell.Close();
            shell = Registry.ClassesRoot.OpenSubKey(@"Directory\shell", true);
            reg_shell(shell);
            shell.Close();
        }
        void http_dereg_shell(object sender, EventArgs e)
        {
            RegistryKey shell = Registry.ClassesRoot.OpenSubKey(@"*\shell", true);
            dereg_shell(shell);
            shell.Close();
            shell = Registry.ClassesRoot.OpenSubKey(@"Directory\shell", true);
            dereg_shell(shell);
            shell.Close();
        }
        void httpd_dir_click(object sender, EventArgs evt)
        {
            TextBox tb = (TextBox)sender;
            string p = tb.Text;
            if (File.Exists(p)) httpd.del_file(p);
            else if (Directory.Exists(p)) httpd.del_dir(p);
            httpd_phash.Remove(tb);
            fp_httpd.Controls.Remove(tb);
            httpd_saveconf();
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
            tb.Width = fp_httpd.Width - 5;
            tb.Anchor = (AnchorStyles.Top | AnchorStyles.Left | AnchorStyles.Right);
            tb.DoubleClick += new EventHandler(httpd_dir_click);
            fp_httpd.Controls.Add(tb);
            httpd_phash.Add(tb, p);
            if (File.Exists(p)) httpd.add_file(p);
            else if (Directory.Exists(p)) httpd.add_dir(p);
            httpd_saveconf();
        }
        void httpd_readconf()
        {
            string[] paths = rconf.get("http_dirs").Split(';');
            foreach (string path in paths) if (path != "") httpd_add_path(path);
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
        }
    }
}
