using lazebird.rabbit.common;
using lazebird.rabbit.http;
using Microsoft.Win32;
using System;
using System.IO;
using System.IO.Pipes;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rhttpd httpd;
        rlog httpdlog;
        void init_form_http()
        {
            httpd_output.HorizontalScrollbar = true;
            httpd_output.HorizontalExtent = 5000;
            httpdlog = new rlog(httpd_output);
            httpd = new rhttpd(httpd_log_func);
            httpd.init_mime(rconf.get("mime"));
            btn_httpd.Click += new EventHandler(httpd_click);
            btn_http_dir.Click += new EventHandler(httpd_dir_click);
            btn_http_reg.Click += new EventHandler(http_reg_shell);
            btn_http_dereg.Click += new EventHandler(http_dereg_shell);
            init_pipeserver();
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
                httpd.set_root(((TextBox)texthash["http_dir"]).Text);
                httpd.start(int.Parse(((TextBox)texthash["http_port"]).Text));
            }
            else
            {
                ((Button)btnhash["httpd_btn"]).Text = Language.trans("开始");
                httpd.stop();
            }
            saveconf(); // save empty config to restore default config
        }
        void httpd_dir_click(object sender, EventArgs e)
        {
            FolderBrowserDialog dialog = new FolderBrowserDialog();
            dialog.SelectedPath = Environment.CurrentDirectory;
            dialog.Description = "请选择文件路径";
            if (dialog.ShowDialog() == DialogResult.OK)
            {
                text_http_dir.Text = dialog.SelectedPath;
            }
        }
        void init_pipeserver()
        {
            Thread t = new Thread(pipesrv_task);
            t.IsBackground = true;
            t.Start();
        }
        void pipesrv_task()
        {
            try
            {
                NamedPipeServerStream pipeServer = new NamedPipeServerStream("lazebird.rabbit.rabbit", PipeDirection.In, 1, PipeTransmissionMode.Message);
                while (true)
                {
                    pipeServer.WaitForConnection();
                    StreamReader sr = new StreamReader(pipeServer);
                    string msg = sr.ReadLine();
                    if (string.IsNullOrEmpty(msg)) continue;
                    httpd_log_func("I: Read pipe " + msg);
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
    }
}
