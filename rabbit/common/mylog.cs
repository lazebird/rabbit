using System.IO;
using System.Windows.Forms;

namespace rabbit.common
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
        public void write(string msg)
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
        public int write(string msg, int line)
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
