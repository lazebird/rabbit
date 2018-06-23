using System.Collections;

namespace rabbit
{
    class myconf
    {
        public static string read(string key)
        {
            string value = (string)Properties.Settings.Default[key];
            if (value != "")
            {
                return value;
            }
            return get_default(key);
        }
        public static void write(string key, string value)
        {
            Properties.Settings.Default[key] = value;
            Properties.Settings.Default.Save();
            Properties.Settings.Default.Upgrade();
        }
        public static string get_default(string key)
        {
            Hashtable cfg = new Hashtable();
            cfg.Add("addr", "www.mozilla.com");
            cfg.Add("interval", "1000");
            cfg.Add("count", "-1");
            return (string)cfg[key];
        }

    }
}
