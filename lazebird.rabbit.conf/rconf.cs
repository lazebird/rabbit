using System;
using System.Collections;
using System.Windows.Forms;

namespace lazebird.rabbit.conf
{
    public class rconf
    {
        Hashtable datas;        // name - value
        Hashtable autosave;     // name - autosave_flag
        Hashtable texts, btns, cbs; // name - UI elements
        Hashtable get_cbs, set_cbs, click_cbs;      // name - callback
        Func<string, string> init;
        Action<Hashtable> save;
        public rconf(Func<string, string> init, Action<Hashtable> save)
        {
            this.init = init;
            this.save = save;
            datas = Hashtable.Synchronized(new Hashtable());
            autosave = Hashtable.Synchronized(new Hashtable());
            texts = Hashtable.Synchronized(new Hashtable());
            btns = Hashtable.Synchronized(new Hashtable());
            cbs = Hashtable.Synchronized(new Hashtable());
            get_cbs = Hashtable.Synchronized(new Hashtable());
            set_cbs = Hashtable.Synchronized(new Hashtable());
            click_cbs = Hashtable.Synchronized(new Hashtable());
        }
        void save_all()
        {
            foreach (string name in get_cbs.Keys) datas[name] = ((Func<string, string>)get_cbs[name])(name);
            save(datas);
        }
        public bool bind(TextBox tb, string name)
        {
            if (datas.ContainsKey(name)) return false;
            texts[name] = tb;
            datas[name] = tb.Text = init(name);
            return true;
        }
        void btn_click(object sender, EventArgs e)
        {
            Button btn = (Button)sender;
            if (click_cbs.ContainsKey(btn.Name)) ((Action<object, EventArgs>)click_cbs[btn.Name])(btn, e);
            datas[btn.Name] = btn.Text;
            if (autosave.ContainsKey(btn.Name)) save_all();
        }
        public bool bind(Button btn, string name, bool autosave_flag, string defval, Action<object, EventArgs> clickcb) // consider button click problem, not only view
        {
            if (datas.ContainsKey(name)) return false;
            btns[name] = btn;
            click_cbs[name] = clickcb;
            btn.Name = name;
            datas[name] = init(name);
            btn.Click += btn_click;
            if (autosave_flag) autosave[name] = true;
            if (init(name) != defval) clickcb(btn, new EventArgs());
            return true;
        }
        public bool bind(CheckBox cb, string name)
        {
            if (datas.ContainsKey(name)) return false;
            cbs[name] = cb;
            datas[name] = init(name);
            cb.Checked = bool.Parse((string)datas[name]);
            return true;
        }
        public bool bind(TabControl tc, string name)
        {
            if (datas.ContainsKey(name)) return false;
            tc.Name = name;
            datas[name] = init(name);
            tc.SelectedIndex = int.Parse((string)datas[name]);
            return true;
        }
        public bool bind(string name, Func<string, string> getcb, Action<string, string> setcb)
        {
            if (datas.ContainsKey(name)) return false;
            datas[name] = init(name);
            if (getcb != null) get_cbs[name] = getcb;
            if (setcb != null) set_cbs[name] = setcb;
            if (setcb != null) setcb(name, init(name));
            return true;
        }
        string get(string name)
        {
            if (!datas.ContainsKey(name)) throw new ArgumentException();
            if (texts.ContainsKey(name)) return ((TextBox)texts[name]).Text;
            if (btns.ContainsKey(name)) return ((Button)btns[name]).Text;
            if (cbs.ContainsKey(name)) return ((CheckBox)cbs[name]).Checked.ToString();
            if (get_cbs.ContainsKey(name)) return ((Func<string, string>)get_cbs[name])(name);
            return (string)datas[name];
        }
        public int getint(string name)
        {
            return int.Parse(get(name));
        }
        public bool getbool(string name)
        {
            return bool.Parse(get(name));
        }
        public string getstr(string name)
        {
            return get(name);
        }
        public void set(string name, string val)
        {
            datas[name] = val;
            if (texts.ContainsKey(name)) ((TextBox)texts[name]).Text = val;
            if (btns.ContainsKey(name)) ((Button)btns[name]).Text = val;
            if (cbs.ContainsKey(name)) ((CheckBox)cbs[name]).Checked = bool.Parse(val);
            if (set_cbs.ContainsKey(name)) ((Action<string, string>)set_cbs[name])(name, val);
        }
    }
}
