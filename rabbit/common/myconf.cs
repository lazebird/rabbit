using System;
using System.Collections;

namespace rabbit.common
{
    class myconf
    {
        public static string read(string key)   // property.setting must be set to avoid exception
        {
            string value = (string)Properties.Settings.Default[key];
            if (value == "")
            {
                return get_default(key);
            }
            return value;
        }
        public static void write(string key, string value)
        {
            Properties.Settings.Default[key] = value;
            Properties.Settings.Default.Save();
            Properties.Settings.Default.Upgrade();
        }
        private static string get_default(string key) // effective only when settings have been set to ""
        {
            Hashtable cfg = new Hashtable();
            cfg.Add("ping_addr", "www.mozilla.com");
            cfg.Add("ping_timeout", "1000");
            cfg.Add("ping_times", "-1");
            cfg.Add("http_port", "8000");
            cfg.Add("tabs", "0");
            if (cfg[key] != null)
            {
                return (string)cfg[key];
            }
            return "";
        }

    }
}
