# fs

## Description
- implement some advance file system api

## Target

## API
1. public rfs(Action<string> log)
    - Constructor
    - log: log print function
    - fhash: file hash for virtual file system
    - dhash: directory hash for virtual file system

2. public int addfile(string vdir, string rfile)
    - add a file to virtual file system
    - vdir: virtual directory path to be added to
    - rfile: real file path in file system

3. public int delfile(string vfile)
    - delete a file from virtual file system
    - vfile: file path in virtual file system

4. public int adddir(string vdir, string rdir)
    - add a directory to virtual file system
    - vdir: directory path in virtual file system
    - rdir: directory path in file system

5. public int deldir(string vdir)
    - delete a directory in virtual file system
    - vdir: directory path in virtual file system

6. public static void readstream(Stream fs, rqueue q, int maxblksz)
    - read a stream and save to queue
    - fs: input file stream
    - q: resource queue, to store file data
    - maxblksz: max block size read and saved

7. public static void writestream(Stream output, rqueue q, string filename)
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