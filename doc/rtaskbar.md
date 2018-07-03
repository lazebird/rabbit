# rhttpd

## Description

## Target

## API
1. public rtaskbar(Action<string> log)  
Constructor
Log: log output interface

2. public bool set(int state, int cur, int total)  
Set the taskbar status
State: refer to TaskbarProgressBarState, error is 4, red; normal is 2, green
Cur: current progress
Total: overall progress

## Sample
```
rhttpd rhttpd = new rhttpd(rhttpd_log_func);
rhttpd.init_mime(myconf.get("mime"));
rhttpd.set_root(".");
rhttpd.start(8000);
void rhttpd_log_func(string msg)
{
    console.write(msg);
}
```