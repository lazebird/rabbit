using System.Collections;
using System.Net;
using System.Net.Sockets;
using System.Windows.Forms;

namespace lazebird.rabbit.chat
{
    class ss
    {
        public UdpClient uc;
        public IPEndPoint r;
        string ruser;
        ArrayList msglst;
        void show_message(string msg)
        {
            TextBox tb = new TextBox();
            message m = new message(ruser, msg, tb);
            m.show();
            msglst.Add(m);
        }
        void show_notification(string msg)
        {
        }
        bool pkt_proc(byte[] buf)
        {
            return true;
        }
        void send_message(string msg)
        {

        }
    }
}
