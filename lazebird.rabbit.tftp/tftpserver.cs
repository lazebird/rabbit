using lazebird.rabbit.fs;
using lazebird.rabbit.queue;
using System;
using System.Collections;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using static lazebird.rabbit.tftp.tftplib;

namespace lazebird.rabbit.tftp
{
    class tftpserver
    {
        Action<string> log;
        Thread tftpd;
        UdpClient uc;
        Hashtable cht;
        rfs rfs;
        public tftpserver(Action<string> log)
        {
            this.log = log;
            cht = new Hashtable();
            rfs = new rfs(log);
        }
        void send_data_block(rqueue q, int blkno, IPEndPoint to)
        {
            try
            {
                byte[] data;
                while ((data = q.consume()) == null) ;
                byte[] buf = CreateDataPacket(blkno, data);
                uc.SendAsync(buf, buf.Length, to);
            }
            catch (Exception e)
            {
                log("!E: " + e.Message);
            }
        }
        void pkt_handler(int port)
        {
            IPEndPoint remote = new IPEndPoint(IPAddress.Any, port);
            byte[] rcvBuffer = uc.Receive(ref remote);
            Opcodes op = (Opcodes)BitConverter.ToUInt16(rcvBuffer, 0);
            if (op == Opcodes.Read || op == Opcodes.Write)
            {
                int pos = 2;
                while (rcvBuffer[pos] != 0) pos++;
                string filename = Encoding.ASCII.GetString(rcvBuffer, 2, pos);
                if (cht.ContainsKey(remote)) // reset
                {
                    cht.Remove(remote);
                }
                rqueue q = new rqueue(2000); // 2000 * 512, max memory used 1M
                cht.Add(remote, new tftpsession(1, filename, q));
                new Thread(() => rfs.readfile(new FileStream(filename, FileMode.Open, FileAccess.Read), q, 512)).Start();    // 10000000, max block size 10M
                new Thread(() => send_data_block(q, 1, remote)).Start();
            }
            else if (op == Opcodes.Ack)
            {
                tftpsession s = (tftpsession)cht[remote];
                if (BitConverter.ToUInt16(rcvBuffer, 2) == s.blkno)
                {
                    s.blkno++;
                    new Thread(() => send_data_block(s.q, s.blkno, remote)).Start();
                }
            }
            else  // error
            {
                byte[] buf = CreateErrPacket("unknown error");
                uc.SendAsync(buf, buf.Length, remote);
            }
        }
        void server_task(int port)
        {
            try
            {
                uc = new UdpClient(port);
                while (true) pkt_handler(port);

            }
            catch (Exception e)
            {
                log("!E: " + e.Message);
            }
        }
        public void start(int port)
        {
            if (tftpd == null)
            {
                tftpd = new Thread(() => server_task(port));
                tftpd.Start();
            }
        }
        public void stop()
        {
            try
            {
                tftpd.Abort();
                tftpd = null;
                uc.Close();
                uc.Dispose();
            }
            catch (Exception e)
            {
                log("!E: " + e.Message);
            }
        }
    }
}
