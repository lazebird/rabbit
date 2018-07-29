﻿using lazebird.rabbit.fs;
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
    public class rhttpd : IDisposable
    {
        Action<string> log;
        HttpListener httpListener = null;
        Hashtable mimehash;
        Hashtable fhash;
        rfs rfs;
        public rhttpd(Action<string> log)
        {
            this.log = log;
            fhash = new Hashtable();
            fhash.Add("/", new rfile("dir", "", 0, DateTime.Today));
            rfs = new rfs(log);
        }
        public void init_mime(string value)
        {
            mimehash = new Hashtable();
            string[] mimes = value.Split(';');
            foreach (string mime in mimes)
            {
                string[] e = mime.Split(':');
                if (e.Length == 2)
                {
                    mimehash.Add(e[0], e[1]);
                }
            }

        }
        public bool start(int port)
        {
            httpListener = new HttpListener();
            httpListener.AuthenticationSchemes = AuthenticationSchemes.Anonymous;
            httpListener.Prefixes.Clear();
            httpListener.Prefixes.Add(string.Format("http://+:{0}/", port));
            try
            {
                httpListener.Start();
                httpListener.BeginGetContext(new AsyncCallback(Dispather), null);
                return true;
            }
            catch (Exception e)
            {
                log("!E: " + e.Message);
                httpListener.Close();
                return false;
            }
        }
        public void stop()
        {
            Dispose();
        }
        string get_suffix(string fname)
        {
            int idx = fname.LastIndexOf(".");
            if (idx > 0)
            {
                return fname.Substring(idx);
            }
            return "";
        }
        string get_mime(string suffix)
        {
            return (string)mimehash[suffix] ?? (string)mimehash["*"] ?? "";
        }
        void Dispather(IAsyncResult ar)
        {
            try
            {
                HttpListenerContext context = httpListener.EndGetContext(ar);
                HttpListenerRequest request = context.Request;
                HttpListenerResponse response = context.Response;
                response.ContentEncoding = Encoding.UTF8;
                string uri = Uri.UnescapeDataString(request.RawUrl);
                log("I: " + request.HttpMethod + " " + uri);
                byte[] buffer = null;

                if (fhash.ContainsKey(uri) && ((rfile)fhash[uri]).type == "file")
                {
                    Thread t = new Thread(() => file_load(response, uri));
                    t.IsBackground = true;
                    t.Start();
                }
                else if (!fhash.ContainsKey(uri))
                {
                    buffer = Encoding.UTF8.GetBytes("404");
                    response.ContentLength64 = buffer.Length;
                    response.OutputStream.Write(buffer, 0, buffer.Length);
                    response.OutputStream.Close();
                }
                else if (((rfile)fhash[uri]).type == "dir")
                {
                    buffer = Encoding.UTF8.GetBytes(dir_load(uri));
                    response.ContentLength64 = buffer.Length;
                    response.OutputStream.Write(buffer, 0, buffer.Length);
                    response.OutputStream.Close();
                }
                httpListener.BeginGetContext(new AsyncCallback(Dispather), null);
            }
            catch (Exception e)
            {
                log("!E: " + e.Message);
            }
        }
        void file_load(HttpListenerResponse response, string path)
        {
            Stream output = response.OutputStream;
            response.ContentType = get_mime(get_suffix(path));
            //log("Info: response suffix " + get_suffix(uri) + " ContentType " + response.ContentType);
            FileStream fs = new FileStream((string)((rfile)fhash[path]).path, FileMode.Open, FileAccess.Read);
            response.ContentLength64 = fs.Length;
            rqueue q = new rqueue(10); // 10 * 10M, max memory used 100M
            Thread t1 = new Thread(() => rfs.readstream(fs, q, 10000000));    // 10000000, max block size 10M
            t1.IsBackground = true;
            t1.Start();
            rfs.writestream(output, q, fs.Name);
            t1.Abort();
            q.Dispose();
        }
        string dir_load(string path)
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
            Regex rgright = new Regex("^" + path + ".");
            Regex rgwrong = new Regex("^" + path + ".+/.");
            foreach (string key in fhash.Keys)
            {
                if (!rgright.IsMatch(key) || rgwrong.IsMatch(key))
                {
                    continue;
                }
                index += "<tr>" +
                 "<td><a href=" + Uri.EscapeUriString(key) + ">" + key.Substring(path.Length) + " </a></td>" +
                 "<td>" + (((rfile)fhash[key]).size).ToString("###,###") + "</td>" +
                 "<td>" + ((rfile)fhash[key]).tm + "</td>" +
                 "</tr>";
            }
            index += "</tbody></table></body></html>";
            return index;
        }
        string rootpath;
        public void set_root(string path)
        {
            if (path == null || path == "" || this.rootpath == path)
            {
                return;
            }
            this.rootpath = path;
            log("R: " + path);
            fhash = new Hashtable();
            fhash.Add("/", new rfile("dir", "", 0, DateTime.Today));
            DirectoryInfo dir = new DirectoryInfo(path);
            foreach (FileInfo f in dir.GetFiles())
            {
                rfs.addfile(fhash, "/", f.FullName);
            }
            foreach (DirectoryInfo d in dir.GetDirectories())
            {
                rfs.adddir(fhash, "/", d.FullName, true);
            }
        }
        public void add_dir(string path)
        {
            rfs.adddir(fhash, "/", path, true);
        }
        public void del_dir(string path)
        {
            rfs.deldir(fhash, "/", path);
        }
        public void add_file(string path)
        {
            rfs.addfile(fhash, "/", path);
        }
        public void del_file(string path)
        {
            rfs.delfile(fhash, "/", path);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (httpListener != null) httpListener.Close();
            httpListener = null;
            if (!disposing) return;
            mimehash = null;
            fhash = null;
            rfs = null;
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
