# rqueue

## 描述
- 实现了一个临界资源队列，可以用于生产者消费者模型

## 目标

## API
1. public rqueue() / public rqueue(int maxsize) / public rqueue(int maxsize, int timeout)
    - 构造函数
    - maxsize: 队列大小，默认 100
    - timeout: 读写函数超时时间，默认 1000 ms

2. public byte[] consume()  
    - 消耗/读一个资源
    - 返回资源字节数组

3. public int produce(byte[] buf)  
    - 生产/写一个资源
    - buf: 资源字节数组

4. public void stop()
    - 停止队列，表示生产者生产完成

5. public bool has_stopped()
    - 判断队列是否停止，用于判断生产者是否已经完成

## 示例
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