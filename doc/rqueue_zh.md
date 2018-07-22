# 队列

## 描述
- 实现了一个临界资源队列，可以用于生产者消费者模型

## 目标

## API
1. public rqueue(int maxsize)  
    - 构造函数
    - maxsize: 队列大小

2. public byte[] consume()  
    - 消费一个资源
    - 返回一个字节数组

3. public int produce(byte[] buf)  
    - 生产一个资源
    - buf: 资源数据

## 示例
    ```
    public void readfile(Stream fs, rqueue q, int maxblksz)
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
    }
    public void writefile(Stream output, rqueue q, string path, long length)
    {
        DateTime starttm;
        DateTime stoptm;
        starttm = DateTime.Now;
        int totalsize = 0;
        while (true)
        {
            byte[] buffer = q.consume();
            if (buffer == null)
            {
                if (totalsize == length)
                {
                    break;
                }
                continue;
            }
            else
            {
                output.Write(buffer, 0, buffer.Length);
                totalsize += buffer.Length;
            }
        }
        output.Close();
        stoptm = DateTime.Now;
        log("I: " + path.Substring(path.Substring(0, path.Length - 1).LastIndexOf('/') + 1) + " "
            + length + " B/" + (stoptm - starttm).TotalSeconds.ToString("###,###.00") + " s; @"
            + (length / ((stoptm - starttm).TotalSeconds + 1)).ToString("###,###.00") + " Bps");
    }
    ```