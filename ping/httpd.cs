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
            try
            {
                HttpListenerContext context = httpListener.EndGetContext(ar);
                HttpListenerRequest request = context.Request;
                HttpListenerResponse response = context.Response;
                l.write("Info: request method " + request.HttpMethod);
                l.write("Info: request uri " + request.RawUrl);
                string responseString = "<HTML><BODY> Hello world!</BODY></HTML>";
                byte[] buffer = System.Text.Encoding.UTF8.GetBytes(responseString);
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
    }
}
