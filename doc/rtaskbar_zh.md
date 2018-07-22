# rtaskbar

## Description

## Target

## API
1. public rtaskbar(Action<string> log)  
    - 构造函数
    - log：log输出接口

2. public bool set(int state, int cur, int total)  
    - 设置任务栏状态
    - state：参考TaskbarProgressBarState，错误为4，红色；正常为2，绿色
    - cur：当前进度
    - total：总体进度

## Sample
    ```
    rtaskbar rtaskbar = new rtaskbar(rtaskbar_log_func);
    rtaskbar.set(2, 2, 5);
    void rtaskbar_log_func(string msg)
    {
        console.write(msg);
    }
    ```