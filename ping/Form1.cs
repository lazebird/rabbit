using System;
using System.Collections;
using System.Windows.Forms;

namespace rabbit
{
    public partial class Form1 : Form
    {
        myping ping;
        mylog pinglog;
        Hashtable texthash;
        Hashtable btnhash;
        Hashtable formhash;
        public Form1()
        {
            InitializeComponent();
            AcceptButton = btn_ping;
            //CancelButton = Application.Exit(0);
            CheckForIllegalCrossThreadCalls = false;
            pinglog = new mylog(output);
            ping = new myping(pinglog, ping_stop_cb);
            init_elements();
            readconf();
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
            ((Button)btnhash["btn"]).Text = "开始";
        }
        private void init_elements()
        {
            texthash = new Hashtable();
            btnhash = new Hashtable();
            formhash = new Hashtable();
            texthash.Add("addr", text_addr);
            texthash.Add("timeout", text_interval);
            texthash.Add("times", text_count);
            texthash.Add("logfile", text_logpath);
            btnhash.Add("btn", btn_ping);
            formhash.Add("form", this);
        }
        private void readconf()
        {
            foreach (string key in texthash.Keys)
            {
                ((TextBox)texthash[key]).Text = myconf.read(key);
            }
        }
        private void saveconf()
        {
            foreach (string key in texthash.Keys)
            {
                myconf.write(key, ((TextBox)texthash[key]).Text);
            }
        }
        private void button1_Click(object sender, EventArgs evt)
        {
            if (((Button)btnhash["btn"]).Text == "开始")
            {
                try
                {
                    saveconf(); // save empty config to restore default config
                    ((Form)formhash["form"]).Text = ((TextBox)texthash["addr"]).Text;
                    ((Button)btnhash["btn"]).Text = "停止";
                    pinglog.setfile(text_logpath.Text);
                    pinglog.clear();
                    ping.start(((TextBox)texthash["addr"]).Text, int.Parse(((TextBox)texthash["timeout"]).Text), int.Parse(((TextBox)texthash["times"]).Text));
                }
                catch (Exception e)
                {
                    pinglog.write("Error: " + e.Message);
                }
            }
            else
            {
                // stop
                ping.stop();
            }
        }
        private void button2_Click(object sender, EventArgs e)
        {
            SaveFileDialog filename = new SaveFileDialog();
            //filename.InitialDirectory = Application.StartupPath;
            filename.Filter = "文本文件 (*.txt)|*.txt|All files (*.*)|*.*";
            filename.RestoreDirectory = true;
            filename.CreatePrompt = true;
            filename.OverwritePrompt = true;
            if (filename.ShowDialog() == DialogResult.OK)
            {
                ((TextBox)texthash["logfile"]).Text = filename.FileName;
            }
        }
    }
}
