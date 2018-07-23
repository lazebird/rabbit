using lazebird.rabbit.common;
using lazebird.rabbit.tftp;
using System;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rtftpd tftpd;
        rlog tftpdlog;
        void init_form_tftp()
        {
            tftpdlog = new rlog(tftpd_output);
            tftpd = new rtftpd(tftpd_log_func, 8, 3000);
            tftp_dirbtn1.Click += new EventHandler(tftpd_dir1_click);
            tftp_dirbtn2.Click += new EventHandler(tftpd_dir2_click);
            tftp_dirbtn3.Click += new EventHandler(tftpd_dir3_click);
            tftp_dirbtn4.Click += new EventHandler(tftpd_dir4_click);
            tftp_dirbtn5.Click += new EventHandler(tftpd_dir5_click);
            tftpd_btn.Click += new EventHandler(tftpd_click);
        }
        int tftpd_log_func(int id, string msg)
        {
            return tftpdlog.write(id, msg);
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
            if (((Button)btnhash["tftpd_btn"]).Text == Language.trans("开始"))
            {
                ((Button)btnhash["tftpd_btn"]).Text = Language.trans("停止");
                tftpd.start(69);
                foreach (string key in texthash.Keys)
                {
                    if (key.Contains("tftp_dir"))
                    {
                        tftpd.add_dir(((TextBox)texthash[key]).Text);
                    }
                }
            }
            else
            {
                ((Button)btnhash["tftpd_btn"]).Text = Language.trans("开始");
                tftpd.stop();
            }
            saveconf();
        }
    }
}
