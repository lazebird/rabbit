using System;
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
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            Form1 f = new Form1();
            Language.SetLang(Language.Getsetting(), f, typeof(Form1));
            Application.Run(f);
        }
    }
}
