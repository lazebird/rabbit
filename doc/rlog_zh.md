# rlog

## Description

## Target

## API
1. public rlog(ListBox logview)  
    - 构造函数
    - logview：ListBox控件，用于log输出

2. public void setfile(string name)  
    - 设置log文件，log文件输出时会附带时间戳
    - name：log文件名

3. public void write(string msg)/public int write(int line, string msg)   
    - 打印log
    - line：指定log写入行数(通过返回值可以获得当前写入的位置)
    - msg：log信息

4. public void clear()  
    - 清空log，对log文件无效

## Sample
    ```
    ListBox logview = new ListBox();
    rlog log = new rlog(logview);
    log.write("hello world");
    int line = log.write(0, "this line will be overridden");
    log.write(line, "this line will override previous line");
    ```