using lazebird.rabbit.fs;
using System;
using System.Collections;
using System.IO;
using System.Net;
using Tftp.Net;

namespace lazebird.rabbit.tftp
{
    public class rtftpd
    {
        Func<int, string, int> log;
        Hashtable vfhash;
        Hashtable fhash;
        Hashtable loghash;
        Hashtable logtmhash;
        TftpServer server;
        rfs rfs;

        public rtftpd(Func<int, string, int> log)
        {
            this.log = log;
            vfhash = new Hashtable();
            fhash = new Hashtable();
            loghash = new Hashtable();
            logtmhash = new Hashtable();
            rfs = new rfs(rfs_log);
        }
        void rfs_log(string msg)
        {
            log(0, msg);
        }
        public void start(int port)
        {
            try
            {
                server = new TftpServer(port);
                server.OnReadRequest += new TftpServerEventHandler(server_OnReadRequest);
                server.OnWriteRequest += new TftpServerEventHandler(server_OnWriteRequest);
                server.Start();
                log(0, "Listening port " + port);
            }
            catch (Exception e)
            {
                this.log(0, "Exception: " + e.Message);
            }
        }
        public void stop()
        {
            try
            {
                vfhash.Clear();
                fhash.Clear();
                loghash.Clear();
                logtmhash.Clear();
                server.Dispose();
                log(0, "Stopped & Clear All Directory");
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
            int filecnt = rfs.adddir(vfhash, "/", dir, false);
            log(0, "+D: " + dir + " (" + filecnt + " files).");
            foreach (string key in vfhash.Keys)
            {
                string fname = key.Substring(dir.LastIndexOf('/') + 1);
                if (fhash.ContainsKey(fname))
                {
                    log(0, "!F: " + key + " and " + fhash[fname] + " conflicts");
                    fhash.Remove(fname);
                }
                fhash.Add(fname, key);
            }
        }
        public void del_dir(string dir)
        {
            if (dir == null || dir == "")
            {
                return;
            }
            int filecnt = rfs.deldir(vfhash, "/", dir);
            log(0, "-D: " + dir + " (" + filecnt + " files).");
            ArrayList list = new ArrayList();
            foreach (string key in fhash.Keys)
            {
                if (((string)fhash[key]).Length >= dir.Length && ((string)fhash[key]).Substring(0, dir.Length) == dir)
                {
                    list.Add(key);
                }
            }
            foreach (string f in list)
            {
                fhash.Remove(f);
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
            if (!fhash.ContainsKey(transfer.Filename))
            {
                CancelTransfer(transfer, TftpErrorPacket.FileNotFound);
                return;
            }
            FileInfo file = new FileInfo(Path.Combine((string)fhash[transfer.Filename], transfer.Filename));
            transfer.UserContext = client;      // append client info
            StartTransfer(transfer, new FileStream(file.FullName, FileMode.Open));
        }

        void StartTransfer(ITftpTransfer transfer, Stream stream)
        {
            transfer.OnProgress += new TftpProgressHandler(transfer_OnProgress);
            transfer.OnError += new TftpErrorHandler(transfer_OnError);
            transfer.OnFinished += new TftpEventHandler(transfer_OnFinished);
            transfer.Start(stream);
        }

        void CancelTransfer(ITftpTransfer transfer, TftpErrorPacket reason)
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
        void OutputTransferStatus(ITftpTransfer transfer, string message)
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
