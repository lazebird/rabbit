using lazebird.rabbit.ping;
using System;
using System.Drawing;
using System.Net;
using System.Net.NetworkInformation;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rping scan;
        void init_form_scan()
        {
            scan = new rping(scan_log_func);
            btn_scan.Click += new EventHandler(scan_click);
        }
        void scan_log_func(string msg)
        {
        }
        void scan_reply(PingReply reply, object data)
        {
            if (reply != null && reply.Status == IPStatus.Success)
            {
                ((Label)data).BackColor = Color.Green;
            }
        }
        void start_scan()
        {
            try
            {
                string startip = ((TextBox)texthash["scan_ipstart"]).Text;
                int lastbyte = int.Parse(((TextBox)texthash["scan_ipend"]).Text);
                Byte[] ipbytes = IPAddress.Parse(startip).GetAddressBytes();
                for (int i = ipbytes[3]; i <= lastbyte && i < 255; i++)
                {
                    Label lb = new Label();
                    lb.Text = i.ToString();
                    lb.Width = lb.Height = 28;
                    lb.TextAlign = ContentAlignment.MiddleCenter;
                    lb.BackColor = Color.FromArgb(64, 64, 64);
                    lb.ForeColor = Color.White;
                    fp_scan.Controls.Add(lb);
                    ipbytes[3] = (Byte)i;
                    scan.start_async((new IPAddress(ipbytes)).ToString(), 1000, scan_reply, lb);
                }
            }
            catch (Exception) { }
        }
        void stop_scan()
        {
            fp_scan.Controls.Clear();
        }
        void scan_click(object sender, EventArgs e)
        {
            stop_scan();
            start_scan();
            saveconf();
        }
    }
}
