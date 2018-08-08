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
        public message(string user, string content)
        {
            this.user = user;
            this.content = content;
            timestamp = DateTime.Now;
        }
        public void set_status(int status)
        {
            this.status = status;
        }
        public override string ToString()
        {
            return user + " " + timestamp.ToString() + " " + content + " " + status;
        }
    }
}
