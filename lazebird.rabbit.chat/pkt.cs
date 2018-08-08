using System.Text;

namespace lazebird.rabbit.chat
{
    public class pkt
    {
        public string type;
        public string id;
        public string user;
        public string content;
        bool parse_attr(string name, string val)
        {
            switch (name)
            {
                case "type":
                    type = val;
                    break;
                case "id":
                    id = val;
                    break;
                case "user":
                    user = val;
                    break;
                case "content":
                    content = val;
                    break;
                default:
                    return false;
            }
            return true;
        }
        public bool parse(byte[] buf)
        {
            string s = Encoding.Default.GetString(buf);
            string[] attrs = s.Split(';');
            foreach (string attr in attrs)
            {
                string[] vals = attr.Split('=');
                if (vals.Length != 2) continue; // ignore
                parse_attr(vals[0], vals[1]);

            }
            return false;
        }
        public byte[] pack()
        {
            string s = "";
            s += "type=" + type + ";";
            if (!string.IsNullOrEmpty(user)) s += "user=" + user + ";";
            if (!string.IsNullOrEmpty(id)) s += "id=" + id + ";";
            if (!string.IsNullOrEmpty(content)) s += "content=" + content + ";";
            return Encoding.Default.GetBytes(s);
        }
    }
}
