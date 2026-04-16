using System;
using System.Collections;
using System.Net;
using static lazebird.rabbit.tftp.pkt;

namespace lazebird.rabbit.tftp
{
    public class rtftpc
    {
        Func<int, string, int> log;
        Hashtable opts;
        public rtftpc(Func<int, string, int> log, Hashtable opts)
        {
            this.log = log;
            this.opts = opts;
        }
        void slog(string msg) { log(-1, msg); }
        void oper(Opcodes oper, string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            byte[] buf;
            ss s = ss.get_clnt_session(localFile, remoteFile, oper, tftpmode, log, new IPEndPoint(IPAddress.Parse(srvip), srvport), opts);
            s.pkt_proc(null);
            while (true)
            {
                try
                {
                    buf = s.uc.Receive(ref s.r);   // change remote port automatically
                    if (!s.pkt_proc(buf)) break;
                }
                catch (Exception)
                {
                    if (!s.retry()) break;
                }
                s.progress_display();
            }
            s.session_display();
            s.destroy();
        }
        bool is_dirpath(string s)
        {
            return s == "" || s[s.Length - 1] == '/';
        }
        public void get(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            try
            {
                oper(is_dirpath(remoteFile) ? Opcodes.ReadDir : Opcodes.Read, srvip, srvport, remoteFile, localFile, tftpmode);
            }
            catch (Exception e)
            {
                slog("!E: " + e.ToString());
            }
        }
        public void put(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            try
            {
                oper(Opcodes.Write, srvip, srvport, remoteFile, localFile, tftpmode);
            }
            catch (Exception e)
            {
                slog("!E: " + e.ToString());
            }
        }
    }
}
