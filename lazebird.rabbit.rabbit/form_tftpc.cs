using lazebird.rabbit.common;
using lazebird.rabbit.tftp;
using System;
using System.IO;
using System.Threading;
using System.Windows.Forms;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rtftpc tftpc;
        rlog tftpclog;
        string lasttftpcdir = Environment.CurrentDirectory;
        void init_form_tftpc()
        {
            tftpclog = new rlog(tftpc_output);
            btn_tftpclfile.Click += new EventHandler(tftpc_lfile_click);
            btn_tftpcput.Click += new EventHandler(tftpc_put_click);
            btn_tftpcget.Click += new EventHandler(tftpc_get_click);
        }
        int tftpc_log_func(int id, string msg)
        {
            return tftpclog.write(id, msg);
        }
        void tftpc_lfile_click(object sender, EventArgs e)
        {
            OpenFileDialog fd = new OpenFileDialog();
            fd.Filter = "All files (*.*)|*.*";
            fd.RestoreDirectory = true;
            fd.InitialDirectory = lasttftpcdir;
            if (fd.ShowDialog() == DialogResult.OK)
            {
                lasttftpcdir = Path.GetDirectoryName(fd.FileName);
                text_tftpclfile.Text = fd.FileName;
            }
        }
        string tftpc_ip;
        int tftpc_tout;
        int tftpc_retry;
        int tftpc_blksize;
        string tftpc_lfile;
        string tftpc_rfile;
        void tftpc_read_args()
        {
            saveconf();
            tftpc_ip = ((TextBox)texthash["tftpc_addr"]).Text;
            tftpc_tout = int.Parse(((TextBox)texthash["tftpc_timeout"]).Text);
            tftpc_retry = int.Parse(((TextBox)texthash["tftpc_retry"]).Text);
            tftpc_blksize = int.Parse(((TextBox)texthash["tftpc_blksize"]).Text);
            tftpc_lfile = text_tftpclfile.Text;
            tftpc_rfile = text_tftpcrfile.Text;
            tftpc = new rtftpc(tftpc_log_func, tftpc_tout, tftpc_retry, tftpc_blksize);
        }
        void tftpc_get_click(object sender, EventArgs evt)
        {
            try
            {
                tftpc_read_args();
                Thread t = new Thread(() => tftpc.get(tftpc_ip, 69, tftpc_rfile, tftpc_rfile, Modes.octet));
                t.IsBackground = true;
                t.Start();
            }
            catch (Exception e)
            {
                tftpc_log_func(-1, "!E: " + e.ToString());
            }
        }
        void tftpc_put_click(object sender, EventArgs evt)
        {
            try
            {
                tftpc_read_args();
                Thread t = new Thread(() => tftpc.put(tftpc_ip, 69, Path.GetFileName(tftpc_lfile), tftpc_lfile, Modes.octet));
                t.IsBackground = true;
                t.Start();
            }
            catch (Exception e)
            {
                tftpc_log_func(-1, "!E: " + e.ToString());
            }
        }
    }
}
