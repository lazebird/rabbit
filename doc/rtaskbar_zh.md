# rhttpd

## Description

## Target

## API
1. public rtaskbar(Action<string> log)  
构造函数
log：log输出接口

2. public bool set(int state, int cur, int total)  
设置任务栏状态
state：参考TaskbarProgressBarState，错误为4，红色；正常为2，绿色
cur：当前进度
total：总体进度

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