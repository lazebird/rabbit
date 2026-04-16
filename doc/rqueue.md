# queue

## Description
- implement a critical resource queue, support producer-consumer design model

## Target

## API
1. public rqueue() / public rqueue(int maxsize) / public rqueue(int maxsize, int timeout)
    - Constructor
    - maxsize: queue size, default 100
    - timeout: timeout, default 1000 ms

2. public byte[] consume()  
    - consume a resource
    - return a resouce like byte[]

3. public int produce(byte[] buf)  
    - produce a resource
    - buf: resource data

4. public void stop()
    - stop this queue, means the producer has finished

5. public bool has_stopped()
    - judge if a queue has stopped

## Sample
    ```
    public void readstream(Stream fs, rqueue q, int maxblksz)
        {
            byte[] buffer = null;
            BinaryReader binReader = new BinaryReader(fs);
            long left = fs.Length;
            long size = Math.Min(maxblksz, fs.Length);
            while (left > size)
            {
                buffer = binReader.ReadBytes((int)size);
                while (q.produce(buffer) == 0) ;
                left -= size;
            }
            if (left > 0)
            {
                buffer = binReader.ReadBytes((int)left);
                while (q.produce(buffer) == 0) ;
            }
            binReader.Close();
            fs.Close();
            q.stop();
        }
        public void writestream(Stream output, rqueue q, string filename)
        {
            int starttm = Environment.TickCount;
            byte[] buffer;
            while (true)
            {
                if ((buffer = q.consume()) != null)
                {
                    output.Write(buffer, 0, buffer.Length);
                    this.totalsize += buffer.Length;
                }
                else if (q.has_stopped())
                    break;
            }
            output.Close();
            int deltatm = Math.Max(1, (Environment.TickCount - starttm) / 1000);
            log("I: " + filename + " " + totalsize + " B/" + deltatm.ToString("###,###.00") + " s; @" + (totalsize / deltatm).ToString("###,###.00") + " Bps");
        }
    ```