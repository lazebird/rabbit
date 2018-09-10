using System;
using System.Collections;
using System.Drawing;
using System.IO;
using System.Windows.Forms;

namespace lazebird.rabbit.common
{
    public class rpanel : IDisposable
    {
        FlowLayoutPanel fp;
        int width;
        ArrayList list;
        StreamWriter file = null;
        public rpanel(FlowLayoutPanel fp, int width)
        {
            fp.AutoScroll = true;
            this.fp = fp;
            this.width = width;
            list = new ArrayList();
        }
        public void savefile(string name)
        {
            if (file != null) file.Close();
            file = null;
            if (!string.IsNullOrWhiteSpace(name)) file = new StreamWriter(name, true);
        }
        public TextBox add(string msg, MouseEventHandler mousedown, MouseEventHandler mousedoubleclick)
        {
            if (file != null)
            {
                file.WriteLine(DateTime.Now.ToString() + ": " + msg);
                file.Flush();
            }
            TextBox tb = new TextBox();
            tb.ReadOnly = true;
            tb.BackColor = fp.BackColor;
            tb.BorderStyle = BorderStyle.None;
            tb.ForeColor = Color.White;
            tb.Width = width;
            tb.Text = msg;
            if (mousedown != null) tb.MouseDown += mousedown;
            if (mousedoubleclick != null) tb.MouseDoubleClick += mousedoubleclick;
            fp.Controls.Add(tb);
            list.Add(tb);
            return tb;
        }
        public void del(TextBox tb)
        {
            fp.Controls.Remove(tb);
            list.Remove(tb);
            tb.Dispose();
        }
        public void clear()
        {
            fp.Controls.Clear();
            foreach (TextBox tb in list)
            {
                tb.Dispose();
            }
            list.Clear();
        }
        protected virtual void Dispose(bool disposing)
        {
            if (file != null) file.Close();
            file = null;
            if (!disposing) return;
            clear();
            fp.Dispose();
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
