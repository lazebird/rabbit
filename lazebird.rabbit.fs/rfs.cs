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
        public static long readstream(Stream fs, rqueue q, int maxblksz)
        {
            long ret = fs.Length;
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
            return ret - left;
        }
        public static long writestream(Stream output, rqueue q, string filename)
        {
            long ret = 0;
            byte[] buffer;
            try
            {
                while (true)
                {
                    if ((buffer = q.consume()) != null) output.Write(buffer, 0, buffer.Length);
                    else if (q.has_stopped()) break;
                    if (buffer != null) ret += buffer.Length;
                }
            }
            catch (Exception) { }
            output.Close();
            return ret;
        }
        public static void readstream_log(Stream fs, rqueue q, int maxblksz, string filename, Action<string> log)
        {
            int starttm = Environment.TickCount;
            long size = readstream(fs, q, maxblksz);
            int deltatm = Math.Max(1, (Environment.TickCount - starttm) / 1000);
            log("I: Read file " + filename + " size " + size + " time " + deltatm + "; " + (size / deltatm).ToString("###,###.0") + " Bps; ");
        }
        public static void writestream_log(Stream output, rqueue q, string filename, Action<string> log)
        {
            int starttm = Environment.TickCount;
            long size = writestream(output, q, filename);
            int deltatm = Math.Max(1, (Environment.TickCount - starttm) / 1000);
            log("I: Write file " + filename + " size " + size + " time " + deltatm + "; " + (size / deltatm).ToString("###,###.0") + " Bps; ");
        }
    }
}
