using System;
using System.Diagnostics;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    static class Program
    {
        /// <summary>
        /// 应用程序的主入口点。
        /// </summary>
        [STAThread]
        static void Main()
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
                MessageBox.Show("Program is already running");
                Application.Exit();
            }
        }
    }
}
