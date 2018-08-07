using System;

namespace lazebird.rabbit.chat
{
    class message
    {
        string user;
        string content;
        DateTime timestamp;
        int status;
        int logidx;
        public message(string user, string content)
        {
            this.user = user;
            this.content = content;
            timestamp = DateTime.Now;
            logidx = -1;
        }
        public void set_status(int status)
        {
            this.status = status;
        }
        public void show(Func<int, string, int> log)
        {
            logidx = log(logidx, ToString());
        }
        public override string ToString()
        {
            return user + " " + timestamp.ToString() + " " + content + " " + status;
        }
    }
}
