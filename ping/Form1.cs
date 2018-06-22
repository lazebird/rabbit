using System;
using System.Windows.Forms;

namespace ping
{
    public partial class Form1 : Form
    {
        public Form1()
        {
            InitializeComponent();
            AcceptButton = btn_ping;
            //CancelButton = Application.Exit(0);
            CheckForIllegalCrossThreadCalls = false;
            init_params();
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
        private void init_params()
        {
            setvalue("addr", myconf.read("addr"));
            setvalue("timeout", myconf.read("timeout"));
            setvalue("times", myconf.read("times"));
            setvalue("logfile", myconf.read("logfile"));
        }
        private void saveconf()
        {
            myconf.write("addr", getvalue("addr"));
            myconf.write("timeout", getvalue("timeout"));
            myconf.write("times", getvalue("times"));
            myconf.write("logfile", getvalue("logfile"));
        }
        public string getvalue(string name)
        {
            if (name == "addr")
            {
                return text_addr.Text;
            }
            if (name == "timeout")
            {
                return text_interval.Text;
            }
            if (name == "times")
            {
                return text_count.Text;
            }
            if (name == "logfile")
            {
                return text_logpath.Text;
            }
            if (name == "btn")
            {
                return btn_ping.Text;
            }
            return "";
        }
        public void setvalue(string name, string value)
        {
            if (value == "")
            {
                return;
            }
            if (name == "addr")
            {
                text_addr.Text = value;
                text_addr.Refresh();
            }
            if (name == "timeout")
            {
                text_interval.Text = value;
                text_interval.Refresh();
            }
            if (name == "times")
            {
                text_count.Text = value;
                text_count.Refresh();
            }
            if (name == "logfile")
            {
                text_logpath.Text = value;
                text_logpath.Refresh();
            }
            if (name == "btn")
            {
                btn_ping.Text = value;
                btn_ping.Refresh();
            }
            if (name == "form")
            {
                this.Text = value;
                this.Refresh();
            }
        }
        public void screen_print(string msg)
        {
            output.Items.Add(msg);
            output.TopIndex = output.Items.Count - (int)(output.Height / output.ItemHeight);
            output.Refresh();
        }
        public void screen_clear()
        {
            output.Items.Clear();
        }
        string addr;
        int timeout, times;
        private void env_setup()
        {
            saveconf(); // save empty config to restore default config
            addr = text_addr.Text;
            timeout = int.Parse(text_interval.Text);
            times = int.Parse(text_count.Text);
            mylog.setfile(text_logpath.Text);
            setvalue("form", addr);
            setvalue("btn", "停止");
            screen_clear();
        }
        private void button1_Click(object sender, EventArgs evt)
        {
            if (getvalue("btn") == "开始")
            {
                try
                {
                    env_setup();
                    myping.getinstance().start(addr, timeout, times);
                }
                catch (Exception e)
                {
                    mylog.log("Error: " + e.Message);
                }
            }
            else
            {
                // stop
                myping.getinstance().stop();
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
                setvalue("logfile", filename.FileName);
            }
        }
    }
}
