using lazebird.rabbit.common;
using lazebird.rabbit.ping;
using lazebird.rabbit.tftp;
using System;
using System.Drawing;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        myping scan;
        void init_form_scan()
        {
            scan = new myping(scan_log_func);
            btn_scan.Click += new EventHandler(scan_click);
        }
        void scan_log_func(string msg)
        {
        }
        void draw_scan()
        {
            for (int i = 1; i < 255; i++)
            {
                Label lb = new Label();
                lb.Text = i.ToString();
                lb.Width = lb.Height = 28;
                lb.TextAlign = ContentAlignment.MiddleCenter;
                lb.BackColor = Color.Green;
                lb.ForeColor = Color.White;
                fp_scan.Controls.Add(lb);
            }
        }
        private void scan_click(object sender, EventArgs e)
        {
            draw_scan();
        }
    }
}
