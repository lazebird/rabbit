﻿using System.IO;
using System.Windows.Forms;

namespace rabbit
{
    class mylog
    {
        string logfile = null;
        ListBox logview = null;

        public mylog(ListBox logview)
        {
            this.logview = logview;
        }

        public void setfile(string name)
        {
            if (name != "")
            {
                logfile = name;
            }
            else
            {
                logfile = null;
            }
        }
        public void log(string msg)
        {
            if (logfile != null)
            {
                StreamWriter file = new StreamWriter(logfile, true);
                file.WriteLine(msg);
                file.Close();
            }
            if (logview != null)
            {
                logview.Items.Add(msg);
                logview.TopIndex = logview.Items.Count - (int)(logview.Height / logview.ItemHeight);
                logview.Refresh();
            }
        }
        public void clear()
        {
            if (logview != null)
            {
                logview.Items.Clear();
            }
        }
    }
}
