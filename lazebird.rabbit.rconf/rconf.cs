using System;
using System.Collections;
using System.Windows.Forms;

namespace lazebird.rabbit.rconf
{
    public class rconf
    {
        Hashtable datas;
        Func<string, string> init;
        Action save;
        public rconf(Func<string, string> init, Action save)
        {
            this.init = init;
            this.save = save;
            datas = Hashtable.Synchronized(new Hashtable());
        }
        void tb_leave(object sender, EventArgs e)
        {
            TextBox tb = (TextBox)sender;
            datas[tb.Name] = tb.Text;   // save conf from ui 
            save();
        }
        public void bind(TextBox tb, string name)
        {
            tb.Name = name;
            if (!datas.ContainsKey(name)) datas.Add(name, init(name));  // load from conf file/default conf
            tb.Text = (string)datas[name];
            tb.Leave += new EventHandler(tb_leave);
        }
        void btn_click(object sender, EventArgs e)
        {
            Button btn = (Button)sender;
            datas[btn.Name] = btn.Text;   // save conf from ui 
            save();
        }
        public void bind(Button btn, string name)
        {
            btn.Name = name;
            if (!datas.ContainsKey(name)) datas.Add(name, "");
            btn.Text = (string)datas[name];  // load from conf file/default conf
            btn.Click += new EventHandler(btn_click);
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

    }
}
