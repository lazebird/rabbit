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
        Action<string> log;
        HttpListener httpListener;
        Hashtable mimehash;
        Hashtable ftypehash;
        Hashtable fpathhash;
        Hashtable fsizehash;
        Hashtable ftmhash;
        public httpd(Action<string> log)
        {
            this.log = log;
            httpListener = new HttpListener();
            httpListener.AuthenticationSchemes = AuthenticationSchemes.Anonymous;
            ftypehash = new Hashtable();
            fpathhash = new Hashtable();
            fsizehash = new Hashtable();
            ftmhash = new Hashtable();
            ftypehash.Add("/", "dir");
            fsizehash.Add("/", 0);
            ftmhash.Add("/", 0);
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
                log("I: " + request.HttpMethod + uri);
                byte[] buffer = null;

                if (ftypehash.ContainsKey(uri) && ftypehash[uri] == "file")
                {
                    response.ContentType = get_mime(get_suffix(uri));
                    //log("Info: response suffix " + get_suffix(uri) + " ContentType " + response.ContentType);
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
                log("!E: " + e.Message);
            }
        }
        void _adddir(string vpath, DirectoryInfo dir) // vpath: vfs path, dir: real dir
        {
            if (!dir.Exists || dir.Attributes.HasFlag(FileAttributes.System)) // fix crash bug for select a dir E:
            {
                log("!D: " + dir.FullName + ", A: " + dir.Attributes.ToString());
                return;
            }
            vpath = vpath + dir.Name + "/";
            ftypehash.Add(vpath, "dir");
            fpathhash.Add(vpath, dir.FullName);
            fsizehash.Add(vpath, 0);
            ftmhash.Add(vpath, dir.LastWriteTime);
            log("+D: " + vpath);
            foreach (FileInfo f in dir.GetFiles())
            {
                _addfile(vpath, f);
            }
            foreach (DirectoryInfo d in dir.GetDirectories())
            {
                _adddir(vpath, d);
            }
        }
        void _deldir(string vpath, DirectoryInfo dir) // vpath: vfs path, dir: real dir
        {
            if (!dir.Exists || dir.Attributes.HasFlag(FileAttributes.System)) // fix crash bug for select a dir E:
            {
                log("!D: " + dir.FullName + ", A: " + dir.Attributes.ToString());
                return;
            }
            vpath = vpath + dir.Name + "/";
            ArrayList list = new ArrayList();
            foreach (string f in ftypehash.Keys)
            {
                if (f.Length >= vpath.Length && f.Substring(0, vpath.Length) == vpath)
                {
                    list.Add(f);
                }
            }
            foreach (string f in list)
            {
                log("-" + ((ftypehash[f] == "dir") ? "D" : "F") + ": " + f);
                ftypehash.Remove(f);
                fpathhash.Remove(f);
                fsizehash.Remove(f);
                ftmhash.Remove(f);
            }
        }
        void _addfile(string vpath, FileInfo f) // vpath: vfs path, f: real file
        {
            if (!f.Exists || f.Attributes.HasFlag(FileAttributes.System))
            {
                log("!F: " + f.FullName + ", A: " + f.Attributes.ToString());
                return;
            }
            vpath = vpath + f.Name;
            ftypehash.Add(vpath, "file");
            fpathhash.Add(vpath, f.FullName);
            fsizehash.Add(vpath, f.Length);
            ftmhash.Add(vpath, f.LastWriteTime);
            log("+F: " + vpath);
        }
        void _delfile(string vpath, FileInfo f) // vpath: vfs path, f: real file
        {
            if (!f.Exists || f.Attributes.HasFlag(FileAttributes.System))
            {
                log("!F: " + f.FullName + ", A: " + f.Attributes.ToString());
                return;
            }
            vpath = vpath + f.Name;
            ftypehash.Remove(vpath);
            fpathhash.Remove(vpath);
            fsizehash.Remove(vpath);
            ftmhash.Remove(vpath);
            log("-F: " + vpath);
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
        string rootpath;
        public void set_root(string path)
        {
            if (path == null || path == "" || this.rootpath == path)
            {
                return;
            }
            this.rootpath = path;
            log("R: " + path);
            ftypehash = new Hashtable();
            fpathhash = new Hashtable();
            fsizehash = new Hashtable();
            ftmhash = new Hashtable();
            ftypehash.Add("/", "dir");
            fsizehash.Add("/", 0);
            ftmhash.Add("/", 0);
            DirectoryInfo dir = new DirectoryInfo(path);
            foreach (FileInfo f in dir.GetFiles())
            {
                _addfile("/", f);
            }
            foreach (DirectoryInfo d in dir.GetDirectories())
            {
                _adddir("/", d);
            }
        }
        public void add_dir(string path)
        {
            _adddir("/", new DirectoryInfo(path));
        }
        public void del_dir(string path)
        {
            _deldir("/", new DirectoryInfo(path));
        }
        public void add_file(string path)
        {
            _addfile("/", new FileInfo(path));
        }
        public void del_file(string path)
        {
            _delfile("/", new FileInfo(path));
        }
    }
}
