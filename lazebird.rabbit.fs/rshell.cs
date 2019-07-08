using Microsoft.Win32;

namespace lazebird.rabbit.fs
{
    public class rshell
    {
        string appname;
        string apppath;
        string menuname;

        public rshell(string appname, string apppath, string menuname)
        {
            this.appname = appname;
            this.apppath = apppath;
            this.menuname = menuname;
        }
        void reg(string root)
        {
            RegistryKey parent = Registry.ClassesRoot.OpenSubKey(root, true);
            RegistryKey custom = parent.CreateSubKey(appname);
            custom.SetValue("", menuname);
            RegistryKey cmd = custom.CreateSubKey("command");
            cmd.SetValue("", "\"" + apppath + "\"" + " \"%1\"");
            cmd.Close();
            custom.Close();
            parent.Close();
        }
        void dereg(string root)
        {
            RegistryKey parent = Registry.ClassesRoot.OpenSubKey(root, true);
            parent.DeleteSubKeyTree(appname);
            parent.Close();
        }
        public void reg_file()
        {
            reg(@"*\shell");
        }
        public void reg_dir()
        {
            reg(@"Directory\shell");
        }
        public void dereg_file()
        {
            dereg(@"*\shell");
        }
        public void dereg_dir()
        {
            dereg(@"Directory\shell");
        }
        bool exist(string root)
        {
            bool ret = true;
            RegistryKey parent = Registry.ClassesRoot.OpenSubKey(root, false);
            RegistryKey custom = parent.OpenSubKey(appname, false);
            if (custom == null) ret = false;
            else custom.Close();
            parent.Close();
            return ret;
        }
        public bool file_exist()
        {
            return exist(@"*\shell");
        }
        public bool dir_exist()
        {
            return exist(@"Directory\shell");
        }
        public void reg_autostart()
        {
            RegistryKey parent = Registry.LocalMachine.OpenSubKey(@"SOFTWARE\Microsoft\Windows\CurrentVersion\Run", true);
            parent.SetValue(appname, apppath);
            parent.Close();
        }
        public void dereg_autostart()
        {
            RegistryKey parent = Registry.LocalMachine.OpenSubKey(@"SOFTWARE\Microsoft\Windows\CurrentVersion\Run", true);
            parent.DeleteValue(appname);
            parent.Close();
        }
        public bool autostart_exist()
        {
            bool ret;
            RegistryKey parent = Registry.LocalMachine.OpenSubKey(@"SOFTWARE\Microsoft\Windows\CurrentVersion\Run", false);
            ret = parent.GetValue(appname) != null;
            parent.Close();
            return ret;
        }
    }
}
