using System.Collections;

namespace lazebird.rabbit.common
{
    public class ropt
    {
        public static Hashtable parse_opts(string s)
        {
            Hashtable h = new Hashtable();
            string[] attrs = s.Split(';');
            foreach (string attr in attrs)
            {
                if (attr.Length <= 0) continue;
                string[] opts = attr.Split('=');
                if (opts.Length != 2) continue;
                if (h.ContainsKey(opts[0])) h.Remove(opts[0]);
                h.Add(opts[0], opts[1]);
            }
            return h;
        }
    }
}
