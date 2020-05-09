using System;

namespace lazebird.rabbit.chat
{
    public class message
    {
        public string user;
        public string content;
        public int index;
        DateTime timestamp;
        bool sentfail = false;
        public message(string user, string content, int index)
        {
            this.user = user;
            this.content = content;
            this.index = index;
            timestamp = DateTime.Now;
        }
        public void set_status(bool sentfail)
        {
            this.sentfail = sentfail;
        }
        public override string ToString()
        {
            return user + " " + timestamp.ToString("yyyy/M/d HH:mm:ss") + "\r\n" + content + (sentfail ? " (please refresh and start an new chat!)" : "");
        }
    }
}
