using lazebird.rabbit.common;
using lazebird.rabbit.tftp;
using System;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        tftpd tftpd;
        mylog tftpdlog;
        void init_form_tftp()
        {
            tftpdlog = new mylog(tftpd_output);
            tftpd = new tftpd(tftpd_log_func);
            tftp_dirbtn1.Click += new EventHandler(tftpd_dir1_click);
            tftp_dirbtn2.Click += new EventHandler(tftpd_dir2_click);
            tftp_dirbtn3.Click += new EventHandler(tftpd_dir3_click);
            tftp_dirbtn4.Click += new EventHandler(tftpd_dir4_click);
            tftp_dirbtn5.Click += new EventHandler(tftpd_dir5_click);
            tftpd_btn.Text = "Apply";
            tftpd_btn.Click += new EventHandler(tftpd_click);
        }
        int tftpd_log_func(int line, string msg)
        {
            return tftpdlog.write(line, msg);
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
        private void tftpd_click(object sender, EventArgs e)
        {
            saveconf();
            readconf();       // crash?
        }
    }
}
