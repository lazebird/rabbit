﻿using System;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.plan
{
    public partial class rplanform : Form
    {
        string msg;
        int duration;
        Thread t;
        public rplanform(string msg, int duration)
        {
            this.msg = msg;
            this.duration = duration;
            InitializeComponent();
            Text = msg + " " + DateTime.Now;
            label_msg.DoubleClick += label_click;
        }
        protected override void OnLoad(EventArgs e)
        {
            base.OnLoad(e);
            t = new Thread(() => msg_task());
            t.IsBackground = true;
            t.Start();
        }
        void label_click(object sender, EventArgs e)
        {
            duration = 0;
        }
        void msg_task()
        {
            while (duration-- > 0)
            {
                label_msg.Text = msg + " (" + duration + ")";
                Thread.Sleep(1000); // update progress per second
            }
            this.Close();
        }
    }
}
