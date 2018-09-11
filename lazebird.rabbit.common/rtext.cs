using System;
using System.IO;
using System.Windows.Forms;

namespace lazebird.rabbit.common
{
    public class rtext : IDisposable
    {
        StreamWriter file = null;
        RichTextBox rtb;
        public rtext(RichTextBox rtb)
        {
            this.rtb = rtb;
            rtb.WordWrap = true;
        }
        public void savefile(string name)
        {
            if (file != null) file.Close();
            file = null;
            if (!string.IsNullOrWhiteSpace(name)) file = new StreamWriter(name, true);
        }
        public void write(string msg)
        {
            if (file != null)
            {
                file.WriteLine(DateTime.Now.ToString() + ": " + msg);
                file.Flush();
            }
            //rtb.AppendText(msg + "\r\n"); // cause plan no response
            rtb.Text = rtb.Text + msg + "\r\n";
        }
        public void clear()
        {
            rtb.Text = "";
        }
        protected virtual void Dispose(bool disposing)
        {
            if (file != null) file.Close();
            file = null;
            if (!disposing) return;
            clear();
            rtb.Dispose();
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
