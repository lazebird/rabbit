# rlog

## Description

## Target

## API
1. public rlog(ListBox logview)  
Constructor
Logview: ListBox control for log output

2. public void setfile(string name)  
Set the log file, the log file will be output with a timestamp
Name: log file name

3. public void write(string msg)/public int write(int line, string msg)   
Print log
Line: specifies the number of log write lines (the current write position can be obtained by the return value)
Msg: log information

4. public void clear()  
Clear log, invalid for log file

## Sample
```
ListBox logview = new ListBox();
rlog log = new rlog(logview);
log.write("hello world");
int line = log.write(0, "this line will be overridden");
log.write(line, "this line will override previous line");
```