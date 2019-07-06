using System;
using System.Collections;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        public Form1()
        {
            InitializeComponent();
            CheckForIllegalCrossThreadCalls = false;
            init_form_conf();
            init_form_ping();
            init_form_scan();
            init_form_http();
            init_form_tftpd();
            init_form_tftpc();
            init_form_plan();
            init_form_chat();
            init_form_setting();
        }
        protected override void OnLoad(EventArgs e)
        {
            base.OnLoad(e);
            init_conf_bind();
        }
        Hashtable btnhash = new Hashtable();
        Hashtable indexhash = new Hashtable();
        protected override bool ProcessDialogKey(Keys keyData)
        {
            if (keyData == Keys.Escape)
            {
                this.Close();
                return true;
            }
            else if (keyData == Keys.Enter)
            {
                foreach (DictionaryEntry de in indexhash)
                {
                    if ((int)de.Value == tabs.SelectedIndex)
                        if (btnhash.ContainsKey(de.Key))
                            ((Button)btnhash[de.Key]).PerformClick();
                }
            }
            return base.ProcessDialogKey(keyData);
        }
    }
}
