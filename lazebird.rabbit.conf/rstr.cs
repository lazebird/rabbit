using System;

namespace lazebird.rabbit.conf
{
    public class rstr
    {
        string name;
        string val = "unknown";
        Func<string, string> getcb;
        Action<string, string> setcb;
        public rstr(string name, Func<string, string> get, Action<string, string> set)
        {
            this.name = name;
            this.getcb = get;
            this.setcb = set;
        }
        public string get(string name) { if (getcb != null) return getcb(this.name); return val; }
        public void set(string name, string val) { if (setcb != null) setcb(this.name, val); this.val = val; }
    }
}
