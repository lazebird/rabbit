using lazebird.rabbit.common;
using lazebird.rabbit.fs;
using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading;

namespace lazebird.rabbit.http
{
    class ss : IDisposable
    {
        Action<string> log;
        HttpListenerRequest request;
        HttpListenerResponse response;
        rfs rfs;
        string method;
        string uri;
        string args;
        Hashtable opthash;
        rqueue q;
        Thread t;
        bool autoindex;
        bool videoplay;

        public ss(HttpListenerRequest request, HttpListenerResponse response, rfs rfs)
        {
            log = log_func;
            this.request = request;
            this.response = response;
            this.rfs = rfs;
            method = request.HttpMethod;
            try
            {
                string[] s = Uri.UnescapeDataString(request.RawUrl).Split('?');
                uri = s[0];
                args = s.Length > 1 ? s[1] : "";
                opthash = ropt.parse_opts(args);
                if (opthash.ContainsKey("autoindex")) this.autoindex = bool.Parse((string)opthash["autoindex"]);
                if (opthash.ContainsKey("videoplay")) this.videoplay = bool.Parse((string)opthash["videoplay"]);
            }
            catch (Exception) { }
            response.ContentEncoding = Encoding.UTF8;
        }
        public ss(Action<string> log, HttpListenerRequest request, HttpListenerResponse response, rfs rfs) : this(request, response, rfs)
        {
            this.log = log;
        }
        public ss(Action<string> log, HttpListenerRequest request, HttpListenerResponse response, rfs rfs, bool autoindex, bool videoplay) : this(log, request, response, rfs)
        {
            if (!opthash.ContainsKey("autoindex")) this.autoindex = autoindex;
            if (!opthash.ContainsKey("videoplay")) this.videoplay = videoplay;
        }
        void log_func(string msg) { }
        string uri2rpath(string uri)
        {
            if (rfs.fhash.ContainsKey(uri)) return (string)rfs.fhash[uri];
            string firstname = uri;
            string lastname = null;
            while (firstname != null) // longest match
            {
                firstname = firstname.Replace(@"\", @"/"); // fix format problem
                //log("I: uri " + uri + " search " + firstname + " + " + lastname);
                if (rfs.dhash.ContainsKey(firstname)) return (string)rfs.dhash[firstname] + (lastname == null ? "" : ("/" + lastname));
                if (lastname == null) lastname = Path.GetFileName(firstname);
                else lastname = Path.GetFileName(firstname) + "/" + lastname;
                firstname = Path.GetDirectoryName(firstname);
            }
            return null;
        }
        string uri2mime(Hashtable mimehash, string uri)
        {
            string fname = Path.GetFileName(uri);
            string suffix = "";
            int idx = fname.LastIndexOf(".");
            if (idx >= 0)
                suffix = fname.Substring(idx);
            return (string)mimehash[suffix] ?? (string)mimehash["*"] ?? "";
        }
        bool loadvideo(Hashtable mimehash, string path)
        {
            if (!videoplay) return false;
            string mime = uri2mime(mimehash, path);
            if (!mime.Contains("video/")) return false;
            string s = @"
<head>
    <link href=""https://vjs.zencdn.net/7.1.0/video-js.css"" rel=""stylesheet"">
    <!-- If you'd like to support IE8 (for Video.js versions prior to v7) -->
    <script src=""https://vjs.zencdn.net/ie8/ie8-version/videojs-ie8.min.js""></script>
</head>
<body>
    <video id=""my-video"" class=""video-js"" controls preload=""auto"" width=""640"" height=""264"" poster=""MY_VIDEO_POSTER.jpg"" data-setup=""{}"">
        <source src=""$uri?videoplay=false"" type='$mime'>
        <p class=""vjs-no-js"">
            To view this video please enable JavaScript, and consider upgrading to a web browser that
            <a href=""https://videojs.com/html5-video-support/"" target=""_blank"">supports HTML5 video</a>
        </p>
    </video>
    <script src=""https://vjs.zencdn.net/7.1.0/video.js""></script>
</body>
";
            s = s.Replace("$uri", uri);
            s = s.Replace("$mime", mime);
            byte[] buffer = Encoding.UTF8.GetBytes(s);
            response.ContentLength64 = buffer.Length;
            response.OutputStream.Write(buffer, 0, buffer.Length);
            response.OutputStream.Close();
            return true;
        }
        void loadfile(Hashtable mimehash, string path)
        {
            if (loadvideo(mimehash, path)) return;
            Stream output = response.OutputStream;
            response.ContentType = uri2mime(mimehash, path);
            log("I: file " + path + " mime " + response.ContentType);
            FileStream fs = new FileStream(path, FileMode.Open, FileAccess.Read);
            response.ContentLength64 = fs.Length;
            q = new rqueue(10); // 10 * 10M, max memory used 100M
            t = new Thread(() => rfs.readstream(fs, q, 10000000));    // 10000000, max block size 10M
            t.IsBackground = true;
            t.Start();
            rfs.writestream(output, q, fs.Name);
            t.Abort();
            q.Dispose();
        }
        string create_th(string s1, string s2, string s3)
        {
            return "<tr><th>" + s1 + "</th>" + "<th>" + s2 + "</th>" + "<th>" + s3 + "</th></tr>";
        }
        string create_td(string s1, string s2, string s3)
        {
            return "<tr><td>" + s1 + "</td>" + "<td>" + s2 + "</td>" + "<td>" + s3 + "</td></tr>";
        }
        bool loadindex(Hashtable mimehash, string rdir)
        {
            string indexpath;
            if (!autoindex) return false;
            if (rdir != "")
            {
                indexpath = rdir + "index.html";
                if (File.Exists(indexpath)) { loadfile(mimehash, indexpath); return true; }
                indexpath = rdir + "index.htm";
                if (File.Exists(indexpath)) { loadfile(mimehash, indexpath); return true; }
            }
            indexpath = uri + "index.html";
            if (rfs.fhash.ContainsKey(indexpath)) { loadfile(mimehash, (string)rfs.fhash[indexpath]); return true; }
            indexpath = uri + "index.htm";
            if (rfs.fhash.ContainsKey(indexpath)) { loadfile(mimehash, (string)rfs.fhash[indexpath]); return true; }
            return false;
        }
        void loaddir(Hashtable mimehash, string rdir)
        {
            if (uri[uri.Length - 1] != '/') uri += "/"; // fix uri ending
            if (loadindex(mimehash, rdir)) return;    // auto index
            string index = "<html><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"/></head><body><table><tbody>";
            index += create_th("Name", "Size (Bytes)", "Last Modified");
            if (uri != "/") index += create_td("<a href=" + Uri.EscapeUriString(Path.GetDirectoryName(uri + "../").Replace(@"\", @"/")) + ">" + ".." + " </a>", "-", "-");
            if (rdir != "")
            {
                DirectoryInfo dir = new DirectoryInfo(rdir);
                foreach (FileInfo f in dir.GetFiles())
                    index += create_td("<a href=" + Uri.EscapeUriString(uri + f.Name) + ">" + f.Name + " </a>", f.Length.ToString("###,###"), f.LastWriteTime.ToString());
                foreach (DirectoryInfo d in dir.GetDirectories())
                    index += create_td("<a href=" + Uri.EscapeUriString(uri + d.Name + "/") + ">" + d.Name + "/" + " </a>", "-", d.LastWriteTime.ToString());
            }
            Regex rgright = new Regex("^" + uri + ".");
            Regex rgwrong = new Regex("^" + uri + ".+/.");
            foreach (string key in rfs.fhash.Keys)
            {
                if (!rgright.IsMatch(key) || rgwrong.IsMatch(key))
                    continue;
                FileInfo f = new FileInfo((string)rfs.fhash[key]);
                index += create_td("<a href=" + Uri.EscapeUriString(uri + f.Name) + ">" + f.Name + " </a>", f.Length.ToString("###,###"), f.LastWriteTime.ToString());
            }
            foreach (string key in rfs.dhash.Keys)
            {
                if (!rgright.IsMatch(key) || rgwrong.IsMatch(key))
                    continue;
                DirectoryInfo d = new DirectoryInfo((string)rfs.dhash[key]);
                index += create_td("<a href=" + Uri.EscapeUriString(uri + d.Name + "/") + ">" + d.Name + "/" + " </a>", "-", d.LastWriteTime.ToString());
            }
            index += "</tbody></table></body></html>";
            byte[] buf = Encoding.UTF8.GetBytes(index);
            response.ContentLength64 = buf.Length;
            response.OutputStream.Write(buf, 0, buf.Length);
            response.OutputStream.Close();
        }
        void loaderror(string path, int errno)
        {
            byte[] buffer = Encoding.UTF8.GetBytes(errno + ": path " + path + " uri " + uri);
            response.ContentLength64 = buffer.Length;
            response.OutputStream.Write(buffer, 0, buffer.Length);
            response.OutputStream.Close();
        }
        public void proc_req(Hashtable mimehash)
        {
            string path = uri2rpath(uri);
            if (path == null)
                loaderror(path, 404);
            else if (File.Exists(path)) // file
                loadfile(mimehash, path);
            else if (Directory.Exists(path)) // dir
                loaddir(mimehash, path);
            else if (path == "" && rfs.dhash.ContainsKey(uri))
                loaddir(mimehash, path); // v dir
            else
                loaderror(path, 404);
        }
        public void destroy()
        {
            Dispose(true);
        }
        protected virtual void Dispose(bool disposing)
        {
            if (q != null) q.Dispose();
            if (t != null) t.Abort();
            q = null;
            t = null;
            if (!disposing) return;
            request = null;
            response = null;
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
