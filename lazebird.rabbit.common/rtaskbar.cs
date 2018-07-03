using Microsoft.WindowsAPICodePack.Taskbar;
using System;

namespace lazebird.rabbit.common
{
    public class rtaskbar
    {
        Action<string> log;
        bool taskbar_support;

        public rtaskbar(Action<string> log)
        {
            this.log = log;
            this.taskbar_support = true;
        }

        public bool set(int state, int cur, int total)
        {
            if (taskbar_support)
            {
                try
                {
                    set_internal(state, cur, total);
                }
                catch (Exception e)
                {
                    log(e.Message);
                    taskbar_support = false;
                }
            }
            return taskbar_support;
        }
        void set_internal(int state, int cur, int total)
        {
            if (!TaskbarManager.IsPlatformSupported)
            {
                return;
            }
            TaskbarManager.Instance.SetProgressState((TaskbarProgressBarState)state);
            TaskbarManager.Instance.SetProgressValue(cur, total);
        }
    }
}
