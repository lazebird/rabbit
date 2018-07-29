using System;

namespace lazebird.rabbit.fs
{
    public class rfile
    {
        public string type;
        public string path;
        public long size;
        public DateTime tm;

        public rfile()
        {
        }
        public rfile(string type, string path, long size, DateTime tm)
        {
            this.type = type;
            this.path = path;
            this.size = size;
            this.tm = tm;
        }
    }
}
