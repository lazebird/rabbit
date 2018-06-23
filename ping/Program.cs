using System;
using System.Windows.Forms;

namespace rabbit
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
            mylog.setform(f);
            myping.getinstance().setform(f);
            Application.Run(f);
        }
    }
}
