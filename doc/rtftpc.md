# rtftpc

## Description

## Target

## API
1. public rtftpc(Func<int, string, int> log) public rtftpc(Func<int, string, int> log, int timeout, int maxretry, int blksize)
    - Constructor
    - Log: log output interface; the first parameter is the output position, 0 means automatic position, the second parameter is the string to be output, and the return value is the output position; the interface is initially used for peer progress update of ListBox.
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
    void tftpc_read_args()
    {
        saveconf();
        tftpc_ip = ((TextBox)texthash["tftpc_addr"]).Text;
        tftpc_tout = int.Parse(((TextBox)texthash["tftpc_timeout"]).Text);
        tftpc_retry = int.Parse(((TextBox)texthash["tftpc_retry"]).Text);
        tftpc_blksize = int.Parse(((TextBox)texthash["tftpc_blksize"]).Text);
        tftpc_lfile = text_tftpclfile.Text;
        tftpc_rfile = text_tftpcrfile.Text;
        tftpc = new rtftpc(tftpc_log_func, tftpc_tout, tftpc_retry, tftpc_blksize);
    }
    void tftpc_get_click(object sender, EventArgs e)
    {
        tftpc_read_args();
        Thread t = new Thread(() => tftpc.get(tftpc_ip, 69, tftpc_rfile, tftpc_rfile, Modes.octet));
        t.IsBackground = true;
        t.Start();
    }
    void tftpc_put_click(object sender, EventArgs e)
    {
        tftpc_read_args();
        Thread t = new Thread(() => tftpc.put(tftpc_ip, 69, Path.GetFileName(tftpc_lfile), tftpc_lfile, Modes.octet));
        t.IsBackground = true;
        t.Start();
    }
    ```