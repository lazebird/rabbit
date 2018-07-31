using System;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    static class Program
    {
        /// <summary>
        /// 应用程序的主入口点。
        /// </summary>
        [STAThread]
        static void Main(string[] args)
        {
            bool createdNew;
            Mutex instance = new Mutex(true, "lazebird.rabbit.rabbit", out createdNew);
            if (createdNew)
            {
                Application.EnableVisualStyles();
                Application.SetCompatibleTextRenderingDefault(false);
                Form1 f = new Form1();
                Language.SetLang(Language.Getsetting(), f, typeof(Form1));
                new Thread(() => httpd_shell_proc(args, true)).Start();
                Application.Run(f);
            }
            else
            {
                if (args.Length > 0) httpd_shell_proc(args, false);
                Application.Exit();
            }
        }
        static void httpd_shell_proc(string[] args, bool delay)
        {
            if (delay) Thread.Sleep(1000);
            int port = 2222; // internal com udp port
            try
            {
                IPEndPoint r = new IPEndPoint(IPAddress.Parse("127.0.0.1"), port);
                UdpClient uc = new UdpClient();
                byte[] buf;
                foreach (var path in args)
                {
                    buf = Encoding.Default.GetBytes(path);
                    uc.SendAsync(buf, buf.Length, r);
                }
                uc.Close();
            }
            catch (Exception e)
            {
                MessageBox.Show("!E: " + e.ToString());
            }
        }
    }
}
