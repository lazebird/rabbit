using System;
using System.Windows.Forms;

namespace lazebird.rabbit.chat
{
    class message
    {
        string user;
        string content;
        DateTime timestamp;
        int status;
        TextBox tb;
        public message(string user, string content, TextBox tb)
        {
            this.user = user;
            this.content = content;
            timestamp = DateTime.Now;
            this.tb = tb;
        }
        public void set_status(int status)
        {
            this.status = status;
        }
        public void show()
        {
            tb.Text = ToString();
        }
        public override string ToString()
        {
            return user + " " + timestamp.ToString() + " " + content + " " + status;
        }
    }
}
