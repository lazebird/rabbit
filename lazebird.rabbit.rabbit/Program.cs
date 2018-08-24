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
        static Mutex mutex = new Mutex(true, "lazebird.rabbit.rabbit");
        [STAThread]
        static void Main(string[] args)
        {
            if (mutex.WaitOne(TimeSpan.Zero, true))
            {
                Application.EnableVisualStyles();
                Application.SetCompatibleTextRenderingDefault(false);
                Form1 f = new Form1();
                Language.SetLang(Language.Getsetting(), f, typeof(Form1));
                if (args.Length > 0) new Thread(() => httpd_shell_proc(args, true)).Start();
                Application.Run(f);
                mutex.ReleaseMutex();
            }
            else
            {
                if (args.Length > 0) new Thread(() => httpd_shell_proc(args, false)).Start();
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
                    if (path.Length <= 0) continue;
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
