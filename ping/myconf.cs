namespace ping
{
    class myconf
    {
        public static string read(string key)
        {
            return (string)Properties.Settings.Default[key];
        }
        public static void write(string key, string value)
        {
            Properties.Settings.Default[key] = value;
            Properties.Settings.Default.Save();
            Properties.Settings.Default.Upgrade();
        }
    }
}
