using System;
using System.Drawing;
using System.IO;
using System.Windows.Forms;

namespace lazebird.rabbit.common
{
    public class rlog : IDisposable
    {
        ListBox logview = null;
        StreamWriter file = null;

        public rlog(ListBox logview)
        {
            logview.BackColor = Color.FromArgb(64, 64, 64);
            logview.DoubleClick += Logview_DoubleClick;
            this.logview = logview;
        }
        void Logview_DoubleClick(object sender, EventArgs e)
        {
            Clipboard.SetDataObject(logview.SelectedItem.ToString());
        }
        public void savefile(string name)
        {
            if (file != null) file.Close();
            file = null;
            if (name != "") file = new StreamWriter(name, true);
        }
        public void write(string msg)
        {
            if (file != null)
            {
                file.WriteLine(DateTime.Now.ToString() + ": " + msg);
                file.Flush();
            }
            logview.Items.Add(msg);
            logview.TopIndex = logview.Items.Count - (int)(logview.Height / logview.ItemHeight);
            logview.Refresh();
        }
        public void clear()
        {
            logview.Items.Clear();
        }
        public int write(int line, string msg)
        {
            if (line < 0) line = logview.Items.Add(msg);
            else
            {
                logview.Items.Insert(line, msg);
                logview.Items.RemoveAt(line + 1);
            }
            logview.TopIndex = logview.Items.Count - (int)(logview.Height / logview.ItemHeight);
            logview.Refresh();
            return line;
        }
        protected virtual void Dispose(bool disposing)
        {
            if (file != null) file.Close();
            file = null;
            if (!disposing) return;
            logview.Dispose();
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
