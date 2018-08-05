# plan

## Description

## Target

## API
1. public rplan(Action<string> log, string s) public rplan(Action<string> log, DateTime starttm, int cycle, cycleunit unit, string msg)  
    - Constructor
    - Log: log output interface
    - s: create an rplan by a rplan string, format refer to rplan.tostring()
    - starttm: plan start datetime, or remind time
    - cycle: repeat interval
    - unit: cycle unit, such as minute, hour ...
    - msg: prompt message, unique for every plan

2. public void stop()
    - stop an rplan

3. public static List<string> cycleunitlist()
    - return cycle unit string list

4. public static cycleunit str2unit(string s)
    - translate a string to cycle unit

5. public override string ToString()
    - tostring()

## Sample
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