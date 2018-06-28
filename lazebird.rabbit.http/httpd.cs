using lazebird.rabbit.common;
using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Text;
using System.Text.RegularExpressions;

namespace lazebird.rabbit.http
{
    public class httpd
    {
        mylog l;
        HttpListener httpListener;
        string path;
        Hashtable mimehash;
        Hashtable ftypehash;
        Hashtable fpathhash;
        Hashtable fsizehash;
        Hashtable ftmhash;
        public httpd(mylog l)
        {
            this.l = l;
            httpListener = new HttpListener();
            httpListener.AuthenticationSchemes = AuthenticationSchemes.Anonymous;
            ftypehash = new Hashtable();
            fpathhash = new Hashtable();
            fsizehash = new Hashtable();
            ftmhash = new Hashtable();
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
        string get_suffix(string fname)
        {
            return fname.Substring(fname.LastIndexOf("."));
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
                Stream output = response.OutputStream;
                response.ContentEncoding = Encoding.UTF8;
                string uri = Uri.UnescapeDataString(request.RawUrl);
                l.write("Info: request method " + request.HttpMethod + " uri " + uri);
                byte[] buffer = null;

                if (ftypehash.ContainsKey(uri) && ftypehash[uri] == "file")
                {
                    response.ContentType = get_mime(get_suffix(uri));
                    //l.write("Info: response suffix " + get_suffix(uri) + " ContentType " + response.ContentType);
                    FileStream fs = new FileStream((string)fpathhash[uri], FileMode.Open, FileAccess.Read);
                    BinaryReader binReader = new BinaryReader(fs);
                    response.ContentLength64 = fs.Length;
                    long left = fs.Length;
                    long size = Math.Max(200000000, fs.Length); // max memory size 200M, fix memory not enough exception
                    while (left > size)
                    {
                        buffer = binReader.ReadBytes((int)size);
                        output.Write(buffer, 0, buffer.Length);
                        left -= size;
                    }
                    if (left > 0)
                    {
                        buffer = binReader.ReadBytes((int)left);
                        output.Write(buffer, 0, buffer.Length);
                    }
                    binReader.Close();
                    fs.Close();
                    output.Close();
                    httpListener.BeginGetContext(new AsyncCallback(Dispather), null);
                    return;
                }
                else if (!ftypehash.ContainsKey(uri))
                {
                    buffer = Encoding.UTF8.GetBytes("404");
                }
                else if ((string)ftypehash[uri] == "dir")
                {
                    buffer = Encoding.UTF8.GetBytes(gen_dir_index(uri));
                }
                response.ContentLength64 = buffer.Length;
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
            if (!dir.Exists || dir.Attributes.HasFlag(FileAttributes.System)) // fix crash bug for select a dir E:
            {
                l.write("Ommit dir: " + dir.FullName + ",attributes " + dir.Attributes.ToString());
                return;
            }
            string key = dir.FullName.Substring(path.Length).Replace('\\', '/') + "/";
            ftypehash.Add(key, "dir");
            fsizehash.Add(key, 0);
            ftmhash.Add(key, dir.LastWriteTime);
            //l.write("Dir: " + key);
            foreach (FileInfo f in dir.GetFiles())
            {
                key = f.FullName.Substring(path.Length).Replace('\\', '/');
                ftypehash.Add(key, "file");
                fpathhash.Add(key, f.FullName);
                fsizehash.Add(key, f.Length);
                ftmhash.Add(key, f.LastWriteTime);
                //l.write("File: " + key);
            }
            foreach (DirectoryInfo d in dir.GetDirectories())
            {
                readdir(d);
            }
        }
        string gen_dir_index(string dir)
        {
            string index = "<html><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"/></head><body><table><tbody>";
            index += "<tr>" +
                "<th>Name</th>" +
                "<th>Size (Bytes)</th>" +
                "<th>Last Modified</th>" +
                "</tr>";
            index += "<tr>" +
              "<td><a href=" + Uri.EscapeUriString(dir.Substring(0, dir.Substring(0, dir.Length - 1).LastIndexOf('/') + 1)) + ">" + ".." + " </a></td>" +
              "<td>" + "</td>" +
              "<td>" + "</td>" +
              "</tr>";
            Regex rgright = new Regex("^" + dir + ".");
            Regex rgwrong = new Regex("^" + dir + ".+/.");
            foreach (string key in ftypehash.Keys)
            {
                if (!rgright.IsMatch(key) || rgwrong.IsMatch(key))
                {
                    continue;
                }
                index += "<tr>" +
                 "<td><a href=" + Uri.EscapeUriString(key) + ">" + key.Substring(dir.Length) + " </a></td>" +
                 "<td>" + fsizehash[key] + "</td>" +
                 "<td>" + ftmhash[key] + "</td>" +
                 "</tr>";
            }
            index += "</tbody></table></body></html>";
            return index;
        }
        public void set_dir(string path)
        {
            if (this.path == path)
            {
                return;
            }
            if (path == null || path == "" || path == ".")
            {
                path = Directory.GetCurrentDirectory();
            }
            this.path = path;
            l.write("Info: path " + path);
            ftypehash = new Hashtable();
            fpathhash = new Hashtable();
            fsizehash = new Hashtable();
            ftmhash = new Hashtable();
            readdir(new DirectoryInfo(path));
        }
    }
}
