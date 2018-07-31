using lazebird.rabbit.fs;
using lazebird.rabbit.queue;
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
        rqueue q;
        Thread t;

        public ss(HttpListenerRequest request, HttpListenerResponse response, rfs rfs)
        {
            this.log = log_func;
            this.request = request;
            this.response = response;
            this.rfs = rfs;
            this.method = request.HttpMethod;
            this.uri = Uri.UnescapeDataString(request.RawUrl);
            response.ContentEncoding = Encoding.UTF8;
        }
        public ss(Action<string> log, HttpListenerRequest request, HttpListenerResponse response, rfs rfs) : this(request, response, rfs)
        {
            this.log = log;
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
        void loadfile(Hashtable mimehash, string path)
        {
            Stream output = response.OutputStream;
            response.ContentType = uri2mime(mimehash, uri);
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
        void loaddir(string rdir)
        {
            if (uri[uri.Length - 1] != '/') uri += "/"; // fix uri ending
            string index = "<html><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"/></head><body><table><tbody>";
            index += create_th("Name", "Size (Bytes)", "Last Modified");
            if (uri != "/") index += create_td("<a href=" + Uri.EscapeUriString(Path.GetDirectoryName(uri + "../").Replace(@"\", @"/")) + ">" + ".." + " </a>", "-", "-");
            if (rdir != "")
            {
                DirectoryInfo dir = new DirectoryInfo(rdir);
                foreach (FileInfo f in dir.GetFiles())
                    index += create_td("<a href=" + Uri.EscapeUriString(uri + f.Name) + ">" + f.Name + " </a>", f.Length.ToString("###,###"), f.LastWriteTime.ToString());
                foreach (DirectoryInfo d in dir.GetDirectories())
                    index += create_td("<a href=" + Uri.EscapeUriString(uri + d.Name + "/") + ">" + d.Name + " </a>", "-", d.LastWriteTime.ToString());
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
                index += create_td("<a href=" + Uri.EscapeUriString(uri + d.Name + "/") + ">" + d.Name + " </a>", "-", d.LastWriteTime.ToString());
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
                loaddir(path);
            else if (path == "" && rfs.dhash.ContainsKey(uri))
                loaddir(path); // v dir
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
