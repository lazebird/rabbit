using System;

namespace lazebird.rabbit.conf
{
    public class rint
    {
        string name;
        Func<string, int> getcb;
        Action<string, int> setcb;
        public rint(string name, Func<string, int> get, Action<string, int> set)
        {
            this.name = name;
            this.getcb = get;
            this.setcb = set;
        }
        public int get(string name) { if (getcb != null) return getcb(name); return 0; }
        public void set(string name, int val) { if (setcb != null) setcb(name, val); }
    }
}
