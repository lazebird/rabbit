using System;
using System.Collections.Generic;
using System.Windows.Forms;

namespace lazebird.rabbit.key
{
    public class rkey
    {
        Dictionary<string, int> tabs;
        Dictionary<string, Action<Keys>> funcs;
        public int globalindex = 100;
        public rkey()
        {
            tabs = new Dictionary<string, int>();
            funcs = new Dictionary<string, Action<Keys>>();
        }
        public bool exec(int tabindex, Keys k)
        {
            string name = tabindex.ToString() + ":" + k.ToString();
            if (!tabs.ContainsKey(name)) name = globalindex.ToString()+":" + k.ToString();     // match global key
            if (!tabs.ContainsKey(name)) return false;
            funcs[name](k);
            return true;
        }
        public bool bind(int tabindex, Keys k, Action<Keys> f)
        {
            string name = tabindex.ToString() + ":" + k.ToString();
            if (tabs.ContainsKey(name)) return false;
            tabs[name] = tabindex;
            funcs[name] = f;
            return true;
        }
        public bool bind(int tabindex, Keys k, Action<object, EventArgs> f)
        {
            string name = tabindex.ToString() + ":" + k.ToString();
            if (tabs.ContainsKey(name)) return false;
            tabs[name] = tabindex;
            funcs[name] = new Action<Keys>((v) => f(null, null));
            return true;
        }
        public bool bind(int tabindex, Keys k, Action f)
        {
            string name = tabindex.ToString() + ":" + k.ToString();
            if (tabs.ContainsKey(name)) return false;
            tabs[name] = tabindex;
            funcs[name] = new Action<Keys>((v) => f());
            return true;
        }
    }
}
