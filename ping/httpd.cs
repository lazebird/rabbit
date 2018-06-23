using System;
using System.Net;

namespace rabbit
{
    class httpd
    {
        mylog l;
        HttpListener httpListener;
        public httpd(mylog l)
        {
            this.l = l;
            this.httpListener = new HttpListener();
        }

        public bool start(int port)
        {
            httpListener.AuthenticationSchemes = AuthenticationSchemes.Anonymous;
            httpListener.Prefixes.Add(string.Format("http://*:{0}/", port));
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
            HttpListenerContext context = httpListener.EndGetContext(ar);
            HttpListenerRequest request = context.Request;
            HttpListenerResponse response = context.Response;
            httpListener.BeginGetContext(new AsyncCallback(Dispather), null);
        }
    }
}
