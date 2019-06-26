# rtftpc

## Description

## Target

## API
1. public rtftpc(Func<int, string, int> log, Hashtable opts)
    - Constructor
    - Log: log output interface; the first parameter is the output position, 0 means automatic position, the second parameter is the string to be output, and the return value is the output position; the interface is initially used for peer progress update of ListBox.
    - opts:
        - timeout: packet timeout, default 200 ms
        - maxretry: packet retransmition times, default 10
        - blksize: block size, default 1024 bytes

2. public void get(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
    - get a file from tftp server
    - srvip: server ip
    - srvport: server port
    - remoteFile: server file name
    - localFile: local file name to store
    - tftpmode: tftp mode

3. public void put(string srvip, int srvport, string remoteFile, string localFile, Modes tftpmode)
    - put a file to tftp server
    - srvip: server ip
    - srvport: server port
    - remoteFile: server file name
    - localFile: local file path to read
    - tftpmode: tftp mode

## Sample
    ```
    void tftpc_parse_args()
    {
        tftpc_ip = ((TextBox)texthash["tftpc_addr"]).Text;
        tftpc_opts = ropt.parse_opts(text_tftpcopt.Text);
        tftpc_lfile = text_tftpclfile.Text;
        tftpc_rfile = text_tftpcrfile.Text;
        saveconf();
    }
    void tftpc_get_click(object sender, EventArgs evt)
    {
        try
        {
            tftpc_parse_args();
            tftpc = new rtftpc(tftpc_log_func, tftpc_opts);
            Thread t = new Thread(() => tftpc.get(tftpc_ip, 69, tftpc_rfile, tftpc_rfile, Modes.octet));
            t.IsBackground = true;
            t.Start();
        }
        catch (Exception e)
        {
            tftpc_log_func(-1, "!E: " + e.ToString());
        }
    }
    void tftpc_put_click(object sender, EventArgs evt)
    {
        try
        {
            tftpc_parse_args();
            tftpc = new rtftpc(tftpc_log_func, tftpc_opts);
            Thread t = new Thread(() => tftpc.put(tftpc_ip, 69, Path.GetFileName(tftpc_lfile), tftpc_lfile, Modes.octet));
            t.IsBackground = true;
            t.Start();
        }
        catch (Exception e)
        {
            tftpc_log_func(-1, "!E: " + e.ToString());
        }
    }
    ```