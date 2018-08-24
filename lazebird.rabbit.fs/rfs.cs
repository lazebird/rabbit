using System;
using System.Collections;
using System.IO;

namespace lazebird.rabbit.fs
{
    public class rfs
    {
        Action<string> log;
        public Hashtable fhash;
        public Hashtable dhash;
        public rfs(Action<string> log)
        {
            this.log = log;
            fhash = new Hashtable();
            dhash = new Hashtable();
            adddir("/", ""); // add root automatically
        }
        public int addfile(string vdir, string rfile)  // vdir: virtual path, rfile: real path
        {
            if (!dhash.ContainsKey(vdir)) adddir(vdir, ""); // add an empty vdir
            string vfile = vdir + Path.GetFileName(rfile);
            log("+F: " + vfile + (fhash.ContainsKey(vfile) ? " (exists)" : "" + " -- " + rfile));
            if (!fhash.ContainsKey(vfile)) fhash.Add(vfile, rfile);
            return 1;
        }
        public int delfile(string vfile)  // vfile: virtual path
        {
            if (fhash.ContainsKey(vfile)) fhash.Remove(vfile);
            log("-F: " + vfile);
            return 1;
        }
        public int adddir(string vdir, string rdir)
        {
            log("+D: " + vdir + (dhash.ContainsKey(vdir) ? " (exists)" : "" + " -- " + rdir));
            if (dhash.ContainsKey(vdir) && vdir != "") dhash.Remove(vdir); // replace
            if (!dhash.ContainsKey(vdir)) dhash.Add(vdir, rdir);
            return 1;
        }
        public int deldir(string vdir)
        {
            log("-D: " + vdir);
            if (dhash.ContainsKey(vdir)) dhash.Remove(vdir);
            return 1;
        }
        public static void readstream(Stream fs, rqueue q, int maxblksz)
        {
            byte[] buffer = null;
            BinaryReader binReader = new BinaryReader(fs);
            long left = fs.Length;
            try
            {
                while (left > 0)
                {
                    long size = Math.Min(maxblksz, left);
                    buffer = binReader.ReadBytes((int)size);
                    while (!q.has_stopped() && q.produce(buffer) == 0) ;
                    if (q.has_stopped()) break;
                    left -= size;
                }
            }
            catch (Exception) { }
            binReader.Close();  //fs.Close(); // binReader disposed?
            q.stop();
        }
        public static void writestream(Stream output, rqueue q, string filename)
        {
            byte[] buffer;
            try
            {
                while (true)
                {
                    if ((buffer = q.consume()) != null) output.Write(buffer, 0, buffer.Length);
                    else if (q.has_stopped()) break;
                }
            }
            catch (Exception) { }
            output.Close();
        }
    }
}
