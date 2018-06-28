using lazebird.rabbit.common;
using lazebird.rabbit.ping;
using System;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        myping ping;
        mylog pinglog;
        void init_form_ping()
        {
            pinglog = new mylog(ping_output);
            ping = new myping(pinglog, ping_stop_cb);
            btn_ping.Click += new EventHandler(ping_click);
            btn_ping_log.Click += new EventHandler(ping_log_click);
        }
        public void ping_stop_cb()
        {
            ((Form)formhash["form"]).Text = "Rabbit";
            ((Button)btnhash["ping_btn"]).Text = "开始";
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
    }
}
