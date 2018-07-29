using lazebird.rabbit.fs;
using lazebird.rabbit.queue;
using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Text;
using System.Threading;

namespace lazebird.rabbit.http
{
    class ss
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
        string uri2filepath(string uri)
        {
            if (rfs.fhash.ContainsKey(uri)) return (string)rfs.fhash[uri];
            string dname = uri;
            string fname = "";
            while (dname != null) // longest match
            {
                log("I: uri " + uri + " search " + dname + "//" + fname);
                if (rfs.dhash.ContainsKey(dname)) return (string)rfs.dhash[dname];
                fname = Path.GetFileName(dname) + "/" + fname;
                dname = Path.GetDirectoryName(dname);
            }
            if (rfs.dhash.ContainsKey(uri)) return (string)rfs.dhash[uri];
            if (rfs.dhash.ContainsKey(Path.GetDirectoryName(uri))) return (string)rfs.dhash[Path.GetDirectoryName(uri)] + Path.GetFileName(uri);
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
        void loaddir(string path)
        {
            string index = "<html><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"/></head><body><table><tbody>";
            index += "<tr>" +
                "<th>Name</th>" +
                "<th>Size (Bytes)</th>" +
                "<th>Last Modified</th>" +
                "</tr>";
            index += "<tr>" +
              "<td><a href=" + Uri.EscapeUriString(path.Substring(0, path.Substring(0, path.Length - 1).LastIndexOf('/') + 1)) + ">" + ".." + " </a></td>" +
              "<td>" + "</td>" +
              "<td>" + "</td>" +
              "</tr>";
            DirectoryInfo dir = new DirectoryInfo(path);
            foreach (FileInfo f in dir.GetFiles())
                index += "<tr>" +
                 "<td><a href=" + Uri.EscapeUriString(f.FullName) + ">" + f.Name + " </a></td>" +
                 "<td>" + f.Length.ToString("###,###") + "</td>" +
                 "<td>" + f.LastWriteTime + "</td>" +
                 "</tr>";
            foreach (DirectoryInfo d in dir.GetDirectories())
                index += "<tr>" +
                 "<td><a href=" + Uri.EscapeUriString(d.FullName) + ">" + d.Name + " </a></td>" +
                 "<td>" + "-" + "</td>" +
                 "<td>" + d.LastWriteTime + "</td>" +
                 "</tr>";
            index += "</tbody></table></body></html>";
            byte[] buf = Encoding.UTF8.GetBytes(index);
            response.ContentLength64 = buf.Length;
            response.OutputStream.Write(buf, 0, buf.Length);
            response.OutputStream.Close();
        }
        void loaderror(string path, int errno)
        {
            byte[] buffer = Encoding.UTF8.GetBytes(errno + ": " + path);
            response.ContentLength64 = buffer.Length;
            response.OutputStream.Write(buffer, 0, buffer.Length);
            response.OutputStream.Close();
        }
        public void proc_req(Hashtable mimehash)
        {
            string path = uri2filepath(uri);
            if (File.Exists(path)) // file
                loadfile(mimehash, path);
            else if (Directory.Exists(path)) // dir
                loaddir(path);
            else
                loaderror(path, 404);
        }
    }
}
