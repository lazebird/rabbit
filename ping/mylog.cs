using System.IO;

namespace rabbit
{
    class mylog
    {
        static Form1 f;
        public static void setform(Form1 form)
        {
            f = form;
        }
        static string logfile;
        public static void setfile(string name)
        {
            logfile = name;
        }
        public static void log(string msg)
        {
            if (logfile != null && logfile != "")
            {
                StreamWriter file = new StreamWriter(logfile, true);
                file.WriteLine(msg);
                file.Close();
            }
            if (f != null)
            {
                f.screen_print(msg);
            }
        }
    }
}
