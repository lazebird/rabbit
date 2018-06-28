using lazebird.rabbit.ping;
using System;
using System.Collections;
using System.Drawing;
using System.Net;
using System.Net.NetworkInformation;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        myping scan;
        Hashtable lbhash;
        void init_form_scan()
        {
            scan = new myping(scan_log_func);
            lbhash = new Hashtable();
            btn_scan.Click += new EventHandler(scan_click);
        }
        void scan_log_func(string msg)
        {
        }
        void scan_reply(int id, PingReply reply)
        {
            if (reply == null || reply.Status != IPStatus.Success)
            {
                ((Label)lbhash[id]).BackColor = Color.Red;
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
                    lb.BackColor = Color.Green;
                    lb.ForeColor = Color.White;
                    fp_scan.Controls.Add(lb);
                    ipbytes[3] = (Byte)i;
                    int id = scan.start_async((new IPAddress(ipbytes)).ToString(), 1000, scan_reply);
                    lbhash.Add(id, lb);
                }
            }
            catch (Exception e)
            {

            }
        }
        void stop_scan()
        {
            lbhash.Clear();
            fp_scan.Controls.Clear();
        }
        private void scan_click(object sender, EventArgs e)
        {
            stop_scan();
            start_scan();
            saveconf();
        }
    }
}
