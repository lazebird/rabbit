﻿using lazebird.rabbit.conf;
using System;
using System.Collections;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rconf cfg;
        void init_form_conf()
        {
            cfg = new rconf(conf_init, conf_save);
            key.bind(key.globalindex, Keys.Control | Keys.F5, conf_reset);
        }
        void btn_init_cb(object o, string val)
        {
            Button b = (Button)o;
            if (b.Text == "Start") return;
            b.Text = "Start";
            if (b == btn_ping) ping_click(b, new EventArgs());
            if (b == btn_httpd) httpd_click(b, new EventArgs());
            if (b == tftpd_btn) tftpd_click(b, new EventArgs());
        }
        void init_conf_bind()
        {
            cfg.bind(tabs, "tabindex", null, false);
            cfg.bind(text_pingaddr, "ping_addr", null, false);
            cfg.bind(text_pingopt, "ping_opt", null, false);
            cfg.bind(text_http_port, "http_port", null, false);
            cfg.bind(text_httpopt, "http_opt", null, false);
            cfg.bind(text_tftpdopt, "tftpd_opt", null, false);
            cfg.bind(text_scanstart, "scan_ipstart", null, false);
            cfg.bind(text_scanend, "scan_ipend", null, false);
            cfg.bind(text_scanopt, "scan_opt", null, false);
            cfg.bind(text_tftpcaddr, "tftpc_addr", null, false);
            cfg.bind(text_tftpcopt, "tftpc_opt", null, false);
            cfg.bind(text_planopt, "plan_opt", null, false);
            cfg.bind(btn_ping, "ping_btn", btn_init_cb, true);
            cfg.bind(btn_httpd, "http_btn", btn_init_cb, true);
            cfg.bind(tftpd_btn, "tftpd_btn", btn_init_cb, true);
            cfg.bind(text_chatname, "chatname", null, false);
            cfg.bind(text_chatntf, "chatbrdip", null, false);
            cfg.bind(new rstr("http_dirs", httpd_conf_get, httpd_conf_set), "http_dirs", null, false);
            cfg.bind(new rstr("plans", plan_conf_get, plan_conf_set), "plans", null, false);
            cfg.bind(cb_systray, "systray", null, false);
            cfg.bind(cb_autoupdate, "autoupdate", null, false);
            cfg.bind(new rstr("restartprompt", null, null), "restartprompt", null, false);
            cfg.bind(new rstr("tftpd_dirs", tftpd_conf_get, tftpd_conf_set), "tftpd_dirs", null, false);
            cfg.bind(new rstr("tftpd_dir_index", tftpd_conf_get, tftpd_conf_set), "tftpd_dir_index", null, false);      // depend on & must behind tftpd_dirs
            cfg.bind(new rstr("lang", null, null), "lang", null, false);
            cfg.bind(new rstr("mime", null, null), "mime", null, false);
            httpd.init_mime(cfg.getstr("mime"));
        }
        string conf_init(string key)   // property.setting must be set to avoid exception
        {
            return string.IsNullOrEmpty((string)Properties.Settings.Default[key]) ? get_default(key) : (string)Properties.Settings.Default[key];
        }
        void conf_save(Hashtable c)
        {
            foreach (string name in c.Keys) Properties.Settings.Default[name] = c[name];
            Properties.Settings.Default.Save();
            Properties.Settings.Default.Upgrade();
        }
        void conf_reset()
        {
            Properties.Settings.Default.Reset();
            Application.Restart();
            Application.Exit();
        }
        string get_default(string key) // effective only when settings have been set to ""
        {
            Hashtable cfg = new Hashtable();
            cfg.Add("tabindex", "0");
            cfg.Add("ping_addr", "www.mozilla.com");
            cfg.Add("ping_opt", "interval=1000;count=-1;taskbar=true;log=;");
            cfg.Add("http_port", "80");
            cfg.Add("http_opt", "autoindex=false;videoplay=true;");
            cfg.Add("http_dirs", ".;..;");
            cfg.Add("scan_ipstart", "192.168.1.1");
            cfg.Add("scan_ipend", "254");
            cfg.Add("scan_opt", "interval=1000;count=1;stoponloss=false;hideunreachable=false;");
            cfg.Add("tftpd_opt", "timeout=200;retry=30;override=true;qsize=400000;");
            cfg.Add("tftpd_dirs", ".;");
            cfg.Add("tftpd_dir_index", "0");
            cfg.Add("tftpc_addr", "127.0.0.1");
            cfg.Add("tftpc_opt", "timeout=200;retry=10;blksize=1468;");
            cfg.Add("plan_opt", "override=false;");
            cfg.Add("plans", DateTime.Now.ToString("yyyy/M/d HH:mm:ss") + "|0|minute|Have a rest!;");
            cfg.Add("systray", "false");
            cfg.Add("autoupdate", "true");
            cfg.Add("chatname", Environment.UserName + "@" + Environment.MachineName);
            cfg.Add("chatbrdip", "255.255.255.255");
            cfg.Add("restartprompt", "false");
            string mime = "";
            mime += "*:application/octet-stream;";
            mime += ".323:text/h323;";
            mime += ".acx:application/internet-property-stream;";
            mime += ".ai:application/postscript;";
            mime += ".aif:audio/x-aiff;";
            mime += ".aifc:audio/x-aiff;";
            mime += ".aiff:audio/x-aiff;";
            mime += ".asf:video/x-ms-asf;";
            mime += ".asr:video/x-ms-asf;";
            mime += ".asx:video/x-ms-asf;";
            mime += ".au:audio/basic;";
            mime += ".avi:video/x-msvideo;";
            mime += ".axs:application/olescript;";
            mime += ".bas:text/plain;";
            mime += ".bcpio:application/x-bcpio;";
            mime += ".bin:application/octet-stream;";
            mime += ".bmp:image/bmp;";
            mime += ".c:text/plain;";
            mime += ".cat:application/vnd.ms-pkiseccat;";
            mime += ".cdf:application/x-cdf;";
            mime += ".cer:application/x-x509-ca-cert;";
            mime += ".class:application/octet-stream;";
            mime += ".clp:application/x-msclip;";
            mime += ".cmx:image/x-cmx;";
            mime += ".cod:image/cis-cod;";
            mime += ".cpio:application/x-cpio;";
            mime += ".crd:application/x-mscardfile;";
            mime += ".crl:application/pkix-crl;";
            mime += ".crt:application/x-x509-ca-cert;";
            mime += ".csh:application/x-csh;";
            mime += ".css:text/css;";
            mime += ".dcr:application/x-director;";
            mime += ".der:application/x-x509-ca-cert;";
            mime += ".dir:application/x-director;";
            mime += ".dll:application/x-msdownload;";
            mime += ".dms:application/octet-stream;";
            mime += ".doc:application/msword;";
            mime += ".dot:application/msword;";
            mime += ".dvi:application/x-dvi;";
            mime += ".dxr:application/x-director;";
            mime += ".eps:application/postscript;";
            mime += ".etx:text/x-setext;";
            mime += ".evy:application/envoy;";
            mime += ".exe:application/octet-stream;";
            mime += ".fif:application/fractals;";
            mime += ".flr:x-world/x-vrml;";
            mime += ".gif:image/gif;";
            mime += ".gtar:application/x-gtar;";
            mime += ".gz:application/x-gzip;";
            mime += ".h:text/plain;";
            mime += ".hdf:application/x-hdf;";
            mime += ".hlp:application/winhlp;";
            mime += ".hqx:application/mac-binhex40;";
            mime += ".hta:application/hta;";
            mime += ".htc:text/x-component;";
            mime += ".htm:text/html;";
            mime += ".html:text/html;";
            mime += ".htt:text/webviewhtml;";
            mime += ".ico:image/x-icon;";
            mime += ".ief:image/ief;";
            mime += ".iii:application/x-iphone;";
            mime += ".ins:application/x-internet-signup;";
            mime += ".isp:application/x-internet-signup;";
            mime += ".jfif:image/pipeg;";
            mime += ".jpe:image/jpeg;";
            mime += ".jpeg:image/jpeg;";
            mime += ".jpg:image/jpeg;";
            mime += ".js:application/x-javascript;";
            mime += ".latex:application/x-latex;";
            mime += ".lha:application/octet-stream;";
            mime += ".lsf:video/x-la-asf;";
            mime += ".lsx:video/x-la-asf;";
            mime += ".lzh:application/octet-stream;";
            mime += ".m13:application/x-msmediaview;";
            mime += ".m14:application/x-msmediaview;";
            mime += ".m3u:audio/x-mpegurl;";
            mime += ".man:application/x-troff-man;";
            mime += ".mdb:application/x-msaccess;";
            mime += ".me:application/x-troff-me;";
            mime += ".mht:message/rfc822;";
            mime += ".mhtml:message/rfc822;";
            mime += ".mid:audio/mid;";
            mime += ".mkv:video/x-matroska;";
            mime += ".mny:application/x-msmoney;";
            mime += ".mov:video/quicktime;";
            mime += ".movie:video/x-sgi-movie;";
            mime += ".mp2:video/mpeg;";
            mime += ".mp3:audio/mpeg;";
            mime += ".mp4:video/mp4;";
            mime += ".mpa:video/mpeg;";
            mime += ".mpe:video/mpeg;";
            mime += ".mpeg:video/mpeg;";
            mime += ".mpg:video/mpeg;";
            mime += ".mpp:application/vnd.ms-project;";
            mime += ".mpv2:video/mpeg;";
            mime += ".ms:application/x-troff-ms;";
            mime += ".mvb:application/x-msmediaview;";
            mime += ".nws:message/rfc822;";
            mime += ".oda:application/oda;";
            mime += ".p10:application/pkcs10;";
            mime += ".p12:application/x-pkcs12;";
            mime += ".p7b:application/x-pkcs7-certificates;";
            mime += ".p7c:application/x-pkcs7-mime;";
            mime += ".p7m:application/x-pkcs7-mime;";
            mime += ".p7r:application/x-pkcs7-certreqresp;";
            mime += ".p7s:application/x-pkcs7-signature;";
            mime += ".pbm:image/x-portable-bitmap;";
            mime += ".pdf:application/pdf;";
            mime += ".pfx:application/x-pkcs12;";
            mime += ".pgm:image/x-portable-graymap;";
            mime += ".pko:application/ynd.ms-pkipko;";
            mime += ".pma:application/x-perfmon;";
            mime += ".pmc:application/x-perfmon;";
            mime += ".pml:application/x-perfmon;";
            mime += ".pmr:application/x-perfmon;";
            mime += ".pmw:application/x-perfmon;";
            mime += ".pnm:image/x-portable-anymap;";
            mime += ".pot,:application/vnd.ms-powerpoint;";
            mime += ".ppm:image/x-portable-pixmap;";
            mime += ".pps:application/vnd.ms-powerpoint;";
            mime += ".ppt:application/vnd.ms-powerpoint;";
            mime += ".prf:application/pics-rules;";
            mime += ".ps:application/postscript;";
            mime += ".pub:application/x-mspublisher;";
            mime += ".qt:video/quicktime;";
            mime += ".ra:audio/x-pn-realaudio;";
            mime += ".ram:audio/x-pn-realaudio;";
            mime += ".ras:image/x-cmu-raster;";
            mime += ".rgb:image/x-rgb;";
            mime += ".rmi:audio/mid;";
            mime += ".rmvb:video/rmvb;";
            mime += ".roff:application/x-troff;";
            mime += ".rtf:application/rtf;";
            mime += ".rtx:text/richtext;";
            mime += ".scd:application/x-msschedule;";
            mime += ".sct:text/scriptlet;";
            mime += ".setpay:application/set-payment-initiation;";
            mime += ".setreg:application/set-registration-initiation;";
            mime += ".sh:application/x-sh;";
            mime += ".shar:application/x-shar;";
            mime += ".sit:application/x-stuffit;";
            mime += ".snd:audio/basic;";
            mime += ".spc:application/x-pkcs7-certificates;";
            mime += ".spl:application/futuresplash;";
            mime += ".src:application/x-wais-source;";
            mime += ".sst:application/vnd.ms-pkicertstore;";
            mime += ".stl:application/vnd.ms-pkistl;";
            mime += ".stm:text/html;";
            mime += ".svg:image/svg+xml;";
            mime += ".sv4cpio:application/x-sv4cpio;";
            mime += ".sv4crc:application/x-sv4crc;";
            mime += ".swf:application/x-shockwave-flash;";
            mime += ".t:application/x-troff;";
            mime += ".tar:application/x-tar;";
            mime += ".tcl:application/x-tcl;";
            mime += ".tex:application/x-tex;";
            mime += ".texi:application/x-texinfo;";
            mime += ".texinfo:application/x-texinfo;";
            mime += ".tgz:application/x-compressed;";
            mime += ".tif:image/tiff;";
            mime += ".tiff:image/tiff;";
            mime += ".tr:application/x-troff;";
            mime += ".trm:application/x-msterminal;";
            mime += ".tsv:text/tab-separated-values;";
            mime += ".txt:text/plain;";
            mime += ".uls:text/iuls;";
            mime += ".ustar:application/x-ustar;";
            mime += ".vcf:text/x-vcard;";
            mime += ".vrml:x-world/x-vrml;";
            mime += ".wav:audio/x-wav;";
            mime += ".wcm:application/vnd.ms-works;";
            mime += ".wdb:application/vnd.ms-works;";
            mime += ".wks:application/vnd.ms-works;";
            mime += ".wmf:application/x-msmetafile;";
            mime += ".wmv:video/x-ms-wmv;";
            mime += ".wps:application/vnd.ms-works;";
            mime += ".wri:application/x-mswrite;";
            mime += ".wrl:x-world/x-vrml;";
            mime += ".wrz:x-world/x-vrml;";
            mime += ".xaf:x-world/x-vrml;";
            mime += ".xbm:image/x-xbitmap;";
            mime += ".xla:application/vnd.ms-excel;";
            mime += ".xlc:application/vnd.ms-excel;";
            mime += ".xlm:application/vnd.ms-excel;";
            mime += ".xls:application/vnd.ms-excel;";
            mime += ".xlt:application/vnd.ms-excel;";
            mime += ".xlw:application/vnd.ms-excel;";
            mime += ".xof:x-world/x-vrml;";
            mime += ".xpm:image/x-xpixmap;";
            mime += ".xwd:image/x-xwindowdump;";
            mime += ".z:application/x-compress;";
            mime += ".zip:application/zip;";
            cfg.Add("mime", mime);
            return String.IsNullOrEmpty((string)cfg[key]) ? "" : (string)cfg[key];
        }
    }
}
