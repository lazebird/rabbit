using System;
using System.Collections;
using System.Windows.Forms;

namespace lazebird.rabbit.conf
{
    public class rconf
    {
        Hashtable items;        // name - object
        Hashtable types;        // name - type: { textbox, tab, checkbox, button, combobox, rstr, rint }
        Hashtable autosave;     // name - autosave_flag; only effective by set action
        Func<string, string> conf_init_cb;
        Action<Hashtable> conf_save_cb;
        bool is_initing = false;        // no autosave when binding or initing a new item
        public rconf(Func<string, string> initcb, Action<Hashtable> savecb)
        {
            this.conf_init_cb = initcb;
            this.conf_save_cb = savecb;
            items = Hashtable.Synchronized(new Hashtable());
            autosave = Hashtable.Synchronized(new Hashtable());
            types = Hashtable.Synchronized(new Hashtable());
        }
        void save_all()
        {
            Hashtable datas = new Hashtable();
            foreach (string name in items.Keys) datas[name] = get(name);
            conf_save_cb(datas);
        }
        bool bind(object o, string name, Action<object, string> initcb, bool autosave_flag)
        {
            if (items.ContainsKey(name)) return false;
            items[name] = o;
            types[name] = o.GetType();
            autosave[name] = autosave_flag;
            is_initing = true;
            if (initcb != null) initcb(o, conf_init_cb(name));
            is_initing = false;
            return true;
        }
        public bool bind(TextBox b, string name, Action<object, string> initcb, bool autosave_flag)
        {
            b.Text = conf_init_cb(name);
            return bind((object)b, name, initcb, autosave_flag);
        }
        public bool bind(Button b, string name, Action<object, string> initcb, bool autosave_flag)
        {
            b.Text = conf_init_cb(name);
            return bind((object)b, name, initcb, autosave_flag);
        }
        public bool bind(CheckBox b, string name, Action<object, string> initcb, bool autosave_flag)
        {
            b.Checked = bool.Parse(conf_init_cb(name));
            return bind((object)b, name, initcb, autosave_flag);
        }
        public bool bind(TabControl b, string name, Action<object, string> initcb, bool autosave_flag)
        {
            b.SelectedIndex = int.Parse(conf_init_cb(name));
            return bind((object)b, name, initcb, autosave_flag);
        }
        public bool bind(rstr b, string name, Action<object, string> initcb, bool autosave_flag)
        {
            b.set(name, conf_init_cb(name));
            return bind((object)b, name, initcb, autosave_flag);
        }
        public bool bind(rint b, string name, Action<object, string> initcb, bool autosave_flag)
        {
            b.set(name, int.Parse(conf_init_cb(name)));
            return bind((object)b, name, initcb, autosave_flag);
        }
        string get(string name)
        {
            if (!items.ContainsKey(name)) throw new ArgumentException();
            if ((Type)types[name] == typeof(TextBox))
            {
                TextBox b = (TextBox)items[name];
                return b.Text;
            }
            else if ((Type)types[name] == typeof(Button))
            {
                Button b = (Button)items[name];
                return b.Text;
            }
            else if ((Type)types[name] == typeof(CheckBox))
            {
                CheckBox b = (CheckBox)items[name];
                return b.Checked.ToString();
            }
            else if ((Type)types[name] == typeof(TabControl))
            {
                TabControl b = (TabControl)items[name];
                return b.TabIndex.ToString();
            }
            else if ((Type)types[name] == typeof(rstr))
            {
                rstr b = (rstr)items[name];
                return b.get("");
            }
            else if ((Type)types[name] == typeof(rint))
            {
                rint b = (rint)items[name];
                return b.get("").ToString();
            }
            return "";
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
            if (!items.ContainsKey(name)) throw new ArgumentException();
            if ((Type)types[name] == typeof(TextBox))
            {
                TextBox b = (TextBox)items[name];
                b.Text = val;
            }
            else if ((Type)types[name] == typeof(Button))
            {
                Button b = (Button)items[name];
                b.Text = val;
            }
            else if ((Type)types[name] == typeof(CheckBox))
            {
                CheckBox b = (CheckBox)items[name];
                b.Checked = bool.Parse(val);
            }
            else if ((Type)types[name] == typeof(TabControl))
            {
                TabControl b = (TabControl)items[name];
                b.TabIndex = int.Parse(val);
            }
            if (!is_initing && (bool)autosave[name]) save_all();
        }
    }
}
