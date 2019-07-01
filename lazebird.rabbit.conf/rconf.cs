using System;
using System.Collections;
using System.Windows.Forms;

namespace lazebird.rabbit.conf
{
    public class rconf
    {
        Hashtable datas;
        Hashtable autosave;
        Hashtable texts, btns, cbs;
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
        }
        void tb_leave(object sender, EventArgs e)
        {
            TextBox tb = (TextBox)sender;
            datas[tb.Name] = tb.Text;
        }
        public void bind(TextBox tb, string name)
        {
            texts[name] = tb;
            tb.Name = name;
            if (!datas.ContainsKey(name)) datas.Add(name, init(name));
            tb.Text = (string)datas[name];
            tb.Leave += new EventHandler(tb_leave);
        }
        void btn_click(object sender, EventArgs e)
        {
            Button btn = (Button)sender;
            datas[btn.Name] = btn.Text;
            if (autosave.ContainsKey(btn.Name)) save(datas);
        }
        public void bind(Button btn, string name, bool autosave_flag, Action<object, string> callback) // consider button click problem, not only view
        {
            btns[name] = btn;
            btn.Name = name;
            if (!datas.ContainsKey(name)) datas.Add(name, init(name));
            btn.Text = (string)datas[name];
            btn.Click += new EventHandler(btn_click);
            if (autosave_flag && !autosave.ContainsKey(name)) autosave.Add(name, true);
            callback(btn, btn.Text);
        }
        void cb_click(object sender, EventArgs e)
        {
            CheckBox cb = (CheckBox)sender;
            datas[cb.Name] = cb.Checked.ToString();
        }
        public void bind(CheckBox cb, string name)
        {
            cbs[name] = cb;
            cb.Name = name;
            if (!datas.ContainsKey(name)) datas.Add(name, init(name));
            cb.Checked = bool.Parse((string)datas[name]);
            cb.CheckedChanged += new EventHandler(cb_click);
        }
        void tc_changed(object sender, EventArgs e)
        {
            TabControl tc = (TabControl)sender;
            datas[tc.Name] = tc.SelectedIndex.ToString();
        }
        public void bind(TabControl tc, string name)
        {
            tc.Name = name;
            if (!datas.ContainsKey(name)) datas.Add(name, init(name));
            tc.SelectedIndex = int.Parse((string)datas[name]);
            tc.SelectedIndexChanged += new EventHandler(tc_changed);
        }
        public void bind(string name)
        {
            if (!datas.ContainsKey(name)) datas.Add(name, init(name));
        }
        public int getint(string name)
        {
            if (!datas.ContainsKey(name)) return 0;
            return int.Parse((string)datas[name]);
        }
        public bool getbool(string name)
        {
            if (!datas.ContainsKey(name)) return false;
            return bool.Parse((string)datas[name]);
        }
        public string getstr(string name)
        {
            if (!datas.ContainsKey(name)) return "";
            return (string)datas[name];
        }
        public void set(string name, string val)
        {
            datas[name] = val;
            if (texts.ContainsKey(name)) ((TextBox)texts[name]).Text = val;
            if (btns.ContainsKey(name)) ((Button)btns[name]).Text = val;
            if (cbs.ContainsKey(name)) ((CheckBox)cbs[name]).Checked = bool.Parse(val);
        }
    }
}
