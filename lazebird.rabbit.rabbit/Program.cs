using System;
using System.IO;
using System.IO.Pipes;
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
            System.Threading.Mutex instance = new System.Threading.Mutex(true, "lazebird.rabbit.rabbit", out createdNew);
            if (createdNew)
            {
                Application.EnableVisualStyles();
                Application.SetCompatibleTextRenderingDefault(false);
                Form1 f = new Form1();
                Language.SetLang(Language.Getsetting(), f, typeof(Form1));
                Application.Run(f);
            }
            else
            {
                //MessageBox.Show("Program is already running");
            }
            if (args.Length > 0) httpd_shell_proc(args);
            Application.Exit();
        }
        static void httpd_shell_proc(string[] args)
        {
            try
            {
                NamedPipeClientStream pipeClient = new NamedPipeClientStream("localhost", "lazebird.rabbit.rabbit", PipeDirection.Out);
                pipeClient.Connect(3000);
                StreamWriter sw = new StreamWriter(pipeClient);
                foreach (var path in args) sw.WriteLine(path);
                sw.Close();                //pipeClient.Close();    // disposed by sw?
            }
            catch (Exception e)
            {
                MessageBox.Show("!E: " + e.ToString());
            }
        }
    }
}
