using System.IO;
using System.Net;
using System.Net.Sockets;
using static lazebird.rabbit.tftp.tftppkt;

namespace lazebird.rabbit.tftp
{
    class rtftpc
    {
        int timeout = 1000;
        public void get(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
        {
            IPEndPoint serverEP = new IPEndPoint(IPAddress.Parse(srvip), srvport);
            byte[] sndBuffer = new tftppkt(Opcodes.Read, remoteFile, tftpmode.ToString()).pack();
            byte[] rcvBuffer = new byte[516];
            BinaryWriter fileStream = new BinaryWriter(new FileStream(localFile, FileMode.Create, FileAccess.Write, FileShare.Read));
            EndPoint dataEP = (EndPoint)serverEP;
            Socket tftpSocket = new Socket(serverEP.Address.AddressFamily, SocketType.Dgram, ProtocolType.Udp);
            tftpSocket.SendTo(sndBuffer, sndBuffer.Length, SocketFlags.None, serverEP);
            tftpSocket.ReceiveTimeout = timeout;
            int blkno = 1;
            while (true)
            {
                int len = tftpSocket.ReceiveFrom(rcvBuffer, ref dataEP);
                serverEP.Port = ((IPEndPoint)dataEP).Port;
                if (((Opcodes)rcvBuffer[1]) == Opcodes.Error)
                {
                    break;
                }
                if ((((rcvBuffer[2] << 8) & 0xff00) | rcvBuffer[3]) == blkno)
                {
                    fileStream.Write(rcvBuffer, 4, len - 4);
                    sndBuffer = new tftppkt(Opcodes.Ack, blkno++).pack();
                    tftpSocket.SendTo(sndBuffer, sndBuffer.Length, SocketFlags.None, serverEP);
                }
                if (len < 516)
                {
                    break;
                }
            }
            tftpSocket.Close();
            fileStream.Close();
        }
        public void put(string srvip, int srvport, string remoteFile, string localFile, string tftpMode)
        {
            IPEndPoint serverEP = new IPEndPoint(IPAddress.Parse(srvip), srvport);
            byte[] sndBuffer = new tftppkt(Opcodes.Write, remoteFile, tftpMode).pack();
            byte[] rcvBuffer = new byte[516];
            BinaryReader fileStream = new BinaryReader(new FileStream(localFile, FileMode.Open, FileAccess.Read, FileShare.ReadWrite));
            EndPoint dataEP = (EndPoint)serverEP;
            Socket tftpSocket = new Socket(serverEP.Address.AddressFamily, SocketType.Dgram, ProtocolType.Udp);
            tftpSocket.SendTo(sndBuffer, sndBuffer.Length, SocketFlags.None, serverEP);
            tftpSocket.ReceiveTimeout = timeout;
            int len = tftpSocket.ReceiveFrom(rcvBuffer, ref dataEP);
            serverEP.Port = ((IPEndPoint)dataEP).Port;
            int blkno = 0;
            while (true)
            {
                if (((Opcodes)rcvBuffer[1]) == Opcodes.Error)
                {
                    fileStream.Close();
                    tftpSocket.Close();
                }
                if ((((Opcodes)rcvBuffer[1]) == Opcodes.Ack) && (((rcvBuffer[2] << 8) & 0xff00) | rcvBuffer[3]) == blkno)
                {
                    sndBuffer = new tftppkt(Opcodes.Data, ++blkno, fileStream.ReadBytes(512)).pack();
                    tftpSocket.SendTo(sndBuffer, sndBuffer.Length, SocketFlags.None, serverEP);
                }
                if (sndBuffer.Length < 516)
                {
                    break;
                }
                else
                {
                    len = tftpSocket.ReceiveFrom(rcvBuffer, ref dataEP);
                }
            }
            tftpSocket.Close();
            fileStream.Close();
        }
    }
}
