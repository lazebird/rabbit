using rabbit.common;
using System;
using System.Collections;
using System.IO;
using System.Net;

namespace rabbit.http
{
    class httpd
    {
        mylog l;
        HttpListener httpListener;
        string path;
        Hashtable fpathhash;
        Hashtable fsizehash;
        Hashtable ftmhash;
        public httpd(mylog l)
        {
            this.l = l;
            httpListener = new HttpListener();
            httpListener.AuthenticationSchemes = AuthenticationSchemes.Anonymous;
            fpathhash = new Hashtable();
            fsizehash = new Hashtable();
            ftmhash = new Hashtable();
        }
        public bool start(int port)
        {
            set_dir("");
            httpListener.Prefixes.Clear();
            httpListener.Prefixes.Add(string.Format("http://+:{0}/", port));
            httpListener.Start();
            httpListener.BeginGetContext(new AsyncCallback(Dispather), null);
            return true;
        }
        public void stop()
        {
            httpListener.Stop();
        }
        void Dispather(IAsyncResult ar)
        {
            try
            {
                HttpListenerContext context = httpListener.EndGetContext(ar);
                HttpListenerRequest request = context.Request;
                HttpListenerResponse response = context.Response;
                l.write("Info: request method " + request.HttpMethod + " uri " + request.RawUrl);
                byte[] buffer = null;
                if (request.RawUrl == "/")
                {
                    buffer = System.Text.Encoding.UTF8.GetBytes(gen_index());
                }
                else if (fpathhash[request.RawUrl] == null)
                {
                    buffer = System.Text.Encoding.UTF8.GetBytes("404");
                }
                else
                {
                    FileStream fs = new FileStream((string)fpathhash[request.RawUrl], FileMode.Open, FileAccess.Read);
                    BinaryReader binReader = new BinaryReader(fs);
                    buffer = new byte[fs.Length];
                    binReader.Read(buffer, 0, (int)fs.Length);
                    binReader.Close();
                    fs.Close();
                }
                response.ContentLength64 = buffer.Length;
                System.IO.Stream output = response.OutputStream;
                output.Write(buffer, 0, buffer.Length);
                output.Close();
                httpListener.BeginGetContext(new AsyncCallback(Dispather), null);
            }
            catch (Exception e)
            {
                l.write("Error: " + e.Message);
            }
        }
        void readdir(DirectoryInfo dir)
        {
            foreach (FileInfo f in dir.GetFiles())
            {
                string key = f.FullName.Substring(path.Length).Replace('\\', '/');
                fpathhash.Add(key, f.FullName);
                fsizehash.Add(key, f.Length);
                ftmhash.Add(key, f.LastWriteTime);
            }
            foreach (DirectoryInfo d in dir.GetDirectories())
            {
                readdir(d);
            }
        }
        string gen_index()
        {
            string index = "<html><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"/></head><body><table><tbody>";
            index += "<tr>" +
                "<th>Name</th>" +
                "<th>Size (Bytes)</th>" +
                "<th>Last Modified</th>" +
                "</tr>";
            foreach (string key in fpathhash.Keys)
            {
                index += "<tr>" +
                 "<td><a href=" + key + ">" + key + " </a></td>" +
                 "<td>" + fsizehash[key] + "</td>" +
                 "<td>" + ftmhash[key] + "</td>" +
                 "</tr>";
            }
            index += "</tbody></table></body></html>";
            l.write("Index: " + index);
            return index;
        }
        public void set_dir(string path)
        {
            if (path == null || path == "" || path == ".")
            {
                path = Directory.GetCurrentDirectory();
            }
            this.path = path;
            l.write("Info: set dir " + path);
            fpathhash.Clear();
            readdir(new DirectoryInfo(path));
        }
    }
}
