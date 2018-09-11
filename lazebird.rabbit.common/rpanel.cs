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
        int maxcount = 256;
        public rpanel(FlowLayoutPanel fp, int width)
        {
            fp.AutoScroll = true;
            fp.BackColor = Color.FromArgb(64, 64, 64);
            this.fp = fp;
            this.width = width;
            list = new ArrayList();
        }
        public rpanel(FlowLayoutPanel fp) : this(fp, fp.Width - 20)
        {
        }
        public void savefile(string name)
        {
            if (file != null) file.Close();
            file = null;
            if (!string.IsNullOrWhiteSpace(name)) file = new StreamWriter(name, true);
        }
        delegate TextBox tb_invoker();
        public TextBox add(string msg, MouseEventHandler mousedown, MouseEventHandler mousedoubleclick)
        {
            if (fp.InvokeRequired) return (TextBox)fp.Invoke(new tb_invoker(() => add(msg, mousedown, mousedoubleclick)));
            if (file != null)
            {
                file.WriteLine(DateTime.Now.ToString() + ": " + msg);
                file.Flush();
            }
            if (list.Count >= maxcount) del((TextBox)list[0]);
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
            fp.ScrollControlIntoView(tb);
            list.Add(tb);
            return tb;
        }
        public void write(string msg)
        {
            add(msg, null, null);
        }
        public int write(int id, string msg)
        {
            if (id >= 0 && id < list.Count) ((TextBox)list[id]).Text = msg;
            else id = list.IndexOf(add(msg, null, null));
            return id;
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
