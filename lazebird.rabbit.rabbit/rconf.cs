using System;
using System.Collections;

namespace lazebird.rabbit.rabbit
{
    class rconf
    {
        public static string get(string key)   // property.setting must be set to avoid exception
        {
            return string.IsNullOrEmpty((string)Properties.Settings.Default[key]) ? get_default(key) : (string)Properties.Settings.Default[key];
        }
        public static void set(string key, string value)
        {
            Properties.Settings.Default[key] = value;
            Properties.Settings.Default.Save();
            Properties.Settings.Default.Upgrade();
        }
        static string get_default(string key) // effective only when settings have been set to ""
        {
            Hashtable cfg = new Hashtable();
            cfg.Add("tabindex", "0");
            cfg.Add("ping_addr", "www.mozilla.com");
            cfg.Add("ping_opt", "interval=1000;count=-1;taskbar=true;log=;");
            cfg.Add("http_port", "8000");
            cfg.Add("http_dirs", ".;..;");
            cfg.Add("scan_ipstart", "192.168.1.1");
            cfg.Add("scan_ipend", "254");
            cfg.Add("scan_opt", "filter=on;");
            cfg.Add("tftpd_opt", "timeout=200;retry=30;");
            cfg.Add("tftpd_dirs", ".;");
            cfg.Add("tftpd_dir_index", "0");
            cfg.Add("tftpc_addr", "127.0.0.1");
            cfg.Add("tftpc_opt", "timeout=200;retry=10;blksize=1468;");
            cfg.Add("plans", DateTime.Now + "|45|minute|Have a rest!;");
            cfg.Add("systray", "false");
            cfg.Add("chatname", Environment.UserName + "@" + Environment.MachineName);
            cfg.Add("chatbrdip", "255.255.255.255");
            string mime = "";
            mime += ".svg:image/svg+xml;";
            mime += ".html:text/html;";
            mime += ".htm:text/html;";
            mime += ".js:application/x-javascript;";
            mime += ".css:text/css;";
            mime += ".mp4:video/mpeg4;";
            mime += ".mpeg:video/mpg;";
            mime += ".avi:video/avi;";
            mime += ".mp3:audio/mp3;";
            mime += ".mid:audio/mid;";
            mime += ".jpg:application/x-jpg;";
            mime += ".jpeg:image/jpeg;";
            mime += ".img:application/x-img;";
            mime += ".ico:image/x-icon;";
            mime += ".png:image/png;";
            mime += "*:application/octet-stream;";
            cfg.Add("mime", mime);
            return String.IsNullOrEmpty((string)cfg[key]) ? "" : (string)cfg[key];
        }

    }
}
