using System;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        public Form1()
        {
            InitializeComponent();
            CheckForIllegalCrossThreadCalls = false;
            init_form_key();
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
            chat_click(null, null);
        }
    }
}
