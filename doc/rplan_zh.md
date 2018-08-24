# plan

## 描述

## 目标

## API
1. public rplan(Action<string> log, string s) public rplan(Action<string> log, DateTime starttm, int cycle, cycleunit unit, string msg)  
    - 构造函数
    - Log: 日志接口
    - s: 使用rplan字符串构造，格式参考tostring接口
    - starttm: 计划开始时间，或者下次提醒时间
    - cycle: 重复周期间隔
    - unit: 周期单位，比如分，小时，等
    - msg: 提示信息，每个计划唯一

2. public void stop()
    - 停止计划

3. public static List<string> cycleunitlist()
    - 返回重复周期单位字符串列表

4. public static cycleunit str2unit(string s)
    - 将字符串转换为重复周期单位

5. public override string ToString()
    - 重写tostring()

## 示例
    ```
    rplan ui2plan()
    {
        DateTime starttm = DateTime.Parse(dt1_plan.Text + " " + dt2_plan.Text);
        int cycle = int.Parse(text_plancycle.Text);
        cycleunit unit = str2unit(cb_planunit.Text);
        string msg = text_planmsg.Text;
        if (msg == "")
        {
            plan_log_func("I: msg cannot be null or empty!");
            return null;
        }
        return new rplan(plan_log_func, starttm, cycle, unit, msg);
    }
    p.stop();
    ```