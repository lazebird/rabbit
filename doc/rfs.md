# fs

## Description
- implement some advance file system api

## Target

## API
1. public rfs(Action<string> log)
    - Constructor
    - log: log print function

2. public int addfile(Hashtable hs, string vpath, string rpath)
    - add a file to virtual file system in hashtable
    - hs: file system hashtable
    - vpath: file path in virtual file system
    - rpath: real path in file system

3. public int delfile(Hashtable hs, string vpath, string rpath)
    - delete a file from virtual file system
    - hs: file system hashtable
    - vpath: file path in virtual file system
    - rpath: real path in file system

4. public int adddir(Hashtable hs, string vpath, string rpath, bool recursive)
    - add a directory to virtual file system in hashtable
    - hs: file system hashtable
    - vpath: file path in virtual file system
    - rpath: real path in file system
    - recursive: recursive to add sub-directories

5. public int deldir(Hashtable hs, string vpath, string rpath)
    - delete a directory to virtual file system in hashtable
    - hs: file system hashtable
    - vpath: file path in virtual file system
    - rpath: real path in file system

6. public void readstream(Stream fs, rqueue q, int maxblksz)
    - read a stream and save to queue
    - fs: input file stream
    - q: resource queue, to store file data
    - maxblksz: max block size read and saved

7. public void writestream(Stream output, rqueue q, string path)
    - write queue data to stream
    - output: file stream
    - q: resource queue, stored file data
    - path: file path or name, used for log view


## Sample
    ```
    void file_load(HttpListenerResponse response, string path)
    {
        Stream output = response.OutputStream;
        response.ContentType = get_mime(get_suffix(path));
        //log("Info: response suffix " + get_suffix(uri) + " ContentType " + response.ContentType);
        FileStream fs = new FileStream((string)((rfile)fhash[path]).path, FileMode.Open, FileAccess.Read);
        response.ContentLength64 = fs.Length;
        rqueue q = new rqueue(10); // 10 * 10M, max memory used 100M
        new Thread(() => rfs.readstream(fs, q, 10000000)).Start();    // 10000000, max block size 10M
        new Thread(() => rfs.writestream(output, q, fs.Name)).Start();
    }
    ```