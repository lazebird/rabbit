using System;
using System.Collections;
using System.IO;
using System.Net;
using Tftp.Net;

namespace lazebird.rabbit.tftp
{
    public class tftpd
    {
        Func<int, string, int> log;
        Hashtable dirhash;
        Hashtable filehash;
        Hashtable loghash;
        Hashtable logtmhash;
        TftpServer server;

        public tftpd(Func<int, string, int> log)
        {
            this.log = log;
            dirhash = new Hashtable();
            filehash = new Hashtable();
            loghash = new Hashtable();
            logtmhash = new Hashtable();
            try
            {
                server = new TftpServer();
                server.OnReadRequest += new TftpServerEventHandler(server_OnReadRequest);
                server.OnWriteRequest += new TftpServerEventHandler(server_OnWriteRequest);
                server.Start();
            }
            catch (Exception e)
            {
                this.log(0, "Exception: " + e.Message);
            }
        }
        public void add_dir(string dir)
        {
            if (dir == null || dir == "")
            {
                return;
            }
            string abs_dir = Path.GetFullPath(dir);
            if (dirhash.ContainsKey(abs_dir))
            {
                return;
            }
            dirhash.Add(abs_dir, 1);
            int filecnt = 0;
            foreach (FileInfo f in (new DirectoryInfo(abs_dir)).GetFiles())
            {
                if (filehash.ContainsKey(f.Name))
                {
                    log(0, "!F: " + f.Name + " in both " + filehash[f.Name] + " and " + abs_dir);
                    filehash.Remove(f.Name);
                }
                filehash.Add(f.Name, abs_dir);
                filecnt++;
            }
            log(0, "+D: " + abs_dir + " (" + filecnt + " files).");
        }
        public void del_dir(string dir)
        {
            if (dir == "")
            {
                return;
            }
            string abs_dir = Path.GetFullPath(dir);
            if (dirhash.ContainsKey(abs_dir))
            {
                dirhash.Remove(abs_dir);
                ArrayList al = new ArrayList();
                foreach (DictionaryEntry de in filehash)
                {
                    if ((string)de.Value == abs_dir)
                    {
                        al.Add(de.Key);
                    }
                }
                foreach (string key in al)
                {
                    filehash.Remove(key);
                }
                log(0, "-D: " + abs_dir + " (" + al.Count + " files).");
                al = null;
            }
        }
        void server_OnWriteRequest(ITftpTransfer transfer, EndPoint client)
        {
            String file = Path.Combine(Environment.CurrentDirectory, transfer.Filename);    // upload to current dir only
            if (File.Exists(file))
            {
                CancelTransfer(transfer, TftpErrorPacket.FileAlreadyExists);
            }
            else
            {
                transfer.UserContext = client;      // append client info
                StartTransfer(transfer, new FileStream(file, FileMode.CreateNew));
            }
        }
        void server_OnReadRequest(ITftpTransfer transfer, EndPoint client)
        {
            if (!filehash.ContainsKey(transfer.Filename))
            {
                CancelTransfer(transfer, TftpErrorPacket.FileNotFound);
                return;
            }
            FileInfo file = new FileInfo(Path.Combine((string)filehash[transfer.Filename], transfer.Filename));
            transfer.UserContext = client;      // append client info
            StartTransfer(transfer, new FileStream(file.FullName, FileMode.Open));
        }

        private void StartTransfer(ITftpTransfer transfer, Stream stream)
        {
            transfer.OnProgress += new TftpProgressHandler(transfer_OnProgress);
            transfer.OnError += new TftpErrorHandler(transfer_OnError);
            transfer.OnFinished += new TftpEventHandler(transfer_OnFinished);
            transfer.Start(stream);
        }

        private void CancelTransfer(ITftpTransfer transfer, TftpErrorPacket reason)
        {
            OutputTransferStatus(transfer, "!E: " + reason.ErrorMessage);
            transfer.Cancel(reason);
        }

        void transfer_OnError(ITftpTransfer transfer, TftpTransferError error)
        {
            OutputTransferStatus(transfer, "!E: " + error);
        }

        void transfer_OnFinished(ITftpTransfer transfer)
        {
            OutputTransferStatus(transfer, "Success");
        }

        void transfer_OnProgress(ITftpTransfer transfer, TftpTransferProgress progress)
        {
            OutputTransferStatus(transfer, "" + progress);
        }
        private void OutputTransferStatus(ITftpTransfer transfer, string message)
        {
            string key = (EndPoint)transfer.UserContext + "/" + transfer.Filename;
            if (loghash.ContainsKey(key))
            {
                if (Environment.TickCount - (int)logtmhash[key] > 50 || message == "Success" || message.Contains("!E: "))
                {
                    log((int)loghash[key], key + ": " + message);
                    logtmhash[key] = Environment.TickCount;
                }
            }
            else
            {
                int line = log(0, key + ": " + message);
                loghash.Add(key, line);
                logtmhash.Add(key, Environment.TickCount);
            }
            if (message == "Success" || message.Contains("!E: "))
            {
                loghash.Remove(key);
                logtmhash.Remove(key);
            }
        }
    }
}
