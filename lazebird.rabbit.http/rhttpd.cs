using lazebird.rabbit.fs;
using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Threading;

namespace lazebird.rabbit.http
{
    public class rhttpd : IDisposable
    {
        Action<string> log;
        HttpListener hl = null;
        Hashtable mimehash;
        rfs rfs;
        Hashtable opts;
        public rhttpd(Action<string> log)
        {
            this.log = log;
            rfs = new rfs(log_null);
        }
        void log_null(string msg) { }
        public void init_mime(string value)
        {
            mimehash = new Hashtable();
            string[] mimes = value.Split(';');
            foreach (string mime in mimes)
            {
                string[] e = mime.Split(':');
                if (e.Length == 2)
                    mimehash.Add(e[0], e[1]);
            }

        }
        public void start(int port, Hashtable opts)
        {
            this.opts = opts;
            if (hl != null) stop();
            hl = new HttpListener();
            hl.AuthenticationSchemes = AuthenticationSchemes.Anonymous;
            hl.Prefixes.Clear();
            hl.Prefixes.Add(string.Format("http://+:{0}/", port));
            try
            {
                hl.Start();
                hl.BeginGetContext(new AsyncCallback(dispatcher), null);
            }
            catch (Exception e)
            {
                log("!E: " + e.ToString());
                hl.Close();
                hl = null;
            }
        }
        public void stop()
        {
            Dispose(false);
        }
        void session_task(HttpListenerRequest request, HttpListenerResponse response)
        {
            ss s = new ss(log, request, response, rfs, opts);
            s.proc_req(mimehash);
            s.destroy();
        }
        void dispatcher(IAsyncResult ar)
        {
            try
            {
                HttpListenerContext context = hl.EndGetContext(ar);
                Thread t = new Thread(() => session_task(context.Request, context.Response));
                t.IsBackground = true;
                t.Start();
                hl.BeginGetContext(new AsyncCallback(dispatcher), null);
            }
            catch (Exception) { }
        }

        public void add_dir(string path)
        {
            rfs.adddir("/" + Path.GetFileName(path), path);
        }
        public void del_dir(string path)
        {
            rfs.deldir("/" + Path.GetFileName(path));
        }
        public void add_file(string path)
        {
            rfs.addfile("/", path);
        }
        public void del_file(string path)
        {
            rfs.delfile("/" + Path.GetFileName(path));
        }

        protected virtual void Dispose(bool disposing)
        {
            if (hl != null) hl.Close();
            hl = null;
            if (!disposing) return;
            mimehash = null;
            rfs = null;
        }
        public void Dispose()
        {
            Dispose(true);
        }
    }
}
