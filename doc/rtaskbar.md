# rtaskbar

## Description

## Target

## API
1. public rtaskbar(Action<string> log)  
    - Constructor
    - Log: log output interface

2. public bool set(int state, int cur, int total)  
    - Set the taskbar status
    - State: refer to TaskbarProgressBarState, error is 4, red; normal is 2, green
    - Cur: current progress
    - Total: overall progress

## Sample
    ```
    rtaskbar rtaskbar = new rtaskbar(rtaskbar_log_func);
    rtaskbar.set(2, 2, 5);
    void rtaskbar_log_func(string msg)
    {
        console.write(msg);
    }
    ```