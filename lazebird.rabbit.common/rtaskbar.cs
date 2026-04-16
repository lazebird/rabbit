using Microsoft.WindowsAPICodePack.Taskbar;
using System;

namespace lazebird.rabbit.common
{
    public class rtaskbar
    {
        Action<string> log;
        IntPtr handle;
        bool taskbar_support;

        public rtaskbar(Action<string> log, IntPtr handle)
        {
            this.log = log;
            this.handle = handle;
            taskbar_support = true;
        }
        public void reset()
        {
            taskbar_support = true;
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
                    log("!E: " + e.ToString());
                    taskbar_support = false;
                }
            }
            return taskbar_support;
        }
        void set_internal(int state, int cur, int total)
        {
            if (!TaskbarManager.IsPlatformSupported) return;
            TaskbarManager.Instance.SetProgressState((TaskbarProgressBarState)state, handle);
            TaskbarManager.Instance.SetProgressValue(cur, total, handle);
        }
    }
}
