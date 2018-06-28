using lazebird.rabbit.common;
using lazebird.rabbit.http;
using System;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        httpd httpd;
        mylog httpdlog;
        void init_form_http()
        {
            httpdlog = new mylog(httpd_output);
            httpd = new httpd(httpdlog);
            httpd.init_mime(myconf.get("mime"));
            btn_httpd.Click += new EventHandler(httpd_click);
            btn_http_dir.Click += new EventHandler(httpd_dir_click);
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
    }
}
