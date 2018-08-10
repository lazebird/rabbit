using System;

namespace lazebird.rabbit.chat
{
    public class message
    {
        string user;
        string content;
        DateTime timestamp;
        bool sentfail;
        public message(string user, string content)
        {
            this.user = user;
            this.content = content;
            timestamp = DateTime.Now;
        }
        public void set_status(bool sentfail)
        {
            this.sentfail = sentfail;
        }
        public override string ToString()
        {
            return user + " " + timestamp.ToString() + "\r\n" + content + (sentfail ? " !" : "");
        }
        public string toshortstring()
        {
            return content + (sentfail ? " !" : "");
        }
    }
}
