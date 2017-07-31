using System;
using System.Windows.Forms;
using Microsoft.WindowsAPICodePack.Shell;
using Microsoft.WindowsAPICodePack.Taskbar;
using System.Net.NetworkInformation;
using System.Threading;
using System.IO;
using System.Configuration;

namespace ping
{
    public partial class Form1 : Form
    {
        public Form1()
        {
            InitializeComponent();
            AcceptButton = button1;
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
                return textBox1.Text;
            }
            if (name == "timeout")
            {
                return textBox2.Text;
            }
            if (name == "times")
            {
                return textBox3.Text;
            }
            if (name == "logfile")
            {
                return textBox4.Text;
            }
            if (name == "btn")
            {
                return button1.Text;
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
                textBox1.Text = value;
                textBox1.Refresh();
            }
            if (name == "timeout")
            {
                textBox2.Text = value;
                textBox2.Refresh();
            }
            if (name == "times")
            {
                textBox3.Text = value;
                textBox3.Refresh();
            }
            if (name == "logfile")
            {
                textBox4.Text = value;
                textBox4.Refresh();
            }
            if (name == "btn")
            {
                button1.Text = value;
                button1.Refresh();
            }
        }
        public void screen_print(string msg)
        {
            listbox1.Items.Add(msg);
            listbox1.TopIndex = listbox1.Items.Count - (int)(listbox1.Height / listbox1.ItemHeight);
            listbox1.Refresh();
        }
        public void screen_clear()
        {
            listbox1.Items.Clear();
        }
        string addr;
        int timeout, times;
        private void env_setup()
        {
            saveconf(); // save empty config to restore default config
            addr = textBox1.Text;
            timeout = int.Parse(textBox2.Text);
            times = int.Parse(textBox3.Text);
            mylog.setfile(textBox4.Text);
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
