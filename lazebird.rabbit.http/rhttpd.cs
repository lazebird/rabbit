﻿using lazebird.rabbit.fs;
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
        bool autoindex;
        bool videoplay;
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
        void parse_args(Hashtable opts)
        {
            if (opts.ContainsKey("autoindex")) bool.TryParse((string)opts["autoindex"], out autoindex);
            if (opts.ContainsKey("videoplay")) bool.TryParse((string)opts["videoplay"], out videoplay);
        }
        public void start(int port, Hashtable opts)
        {
            parse_args(opts);
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
            ss s = new ss(log, request, response, rfs, autoindex, videoplay);
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

        string rootpath;
        public void set_root(string path)
        {
            if (path == null || path == "" || this.rootpath == path) return;
            this.rootpath = path;
            log("R: " + path);
            //log("I: parent " + Path.GetDirectoryName(path) + " " + Path.GetFileName(path));
            rfs.adddir("/", path);
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
