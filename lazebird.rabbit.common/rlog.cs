using System;
using System.IO;
using System.Windows.Forms;

namespace lazebird.rabbit.common
{
    public class rlog
    {
        ListBox logview = null;
        StreamWriter file;

        public rlog(ListBox logview)
        {
            this.logview = logview;
        }

        public void setfile(string name)
        {
            if (file != null)
            {
                file.Close();
                file = null;
            }
            if (name != "")
            {
                file = new StreamWriter(name, true);
            }
        }
        public void write(string msg)
        {
            if (file != null)
            {
                file.WriteLine(DateTime.Now.ToString() + ": " + msg);
                file.Flush();
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
        public int write(int line, string msg)
        {
            if (logview == null)
            {
                return 0;
            }
            if (line == 0)
            {
                line = logview.Items.Add(msg);
                logview.TopIndex = logview.Items.Count - (int)(logview.Height / logview.ItemHeight);
            }
            else
            {
                logview.Items.Insert(line, msg);
                logview.Items.RemoveAt(line + 1);
                logview.TopIndex = logview.Items.Count - (int)(logview.Height / logview.ItemHeight);
            }
            logview.Refresh();
            return line;
        }
    }
}
