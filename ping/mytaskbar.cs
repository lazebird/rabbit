using Microsoft.WindowsAPICodePack.Taskbar;
using System;

namespace rabbit
{
    class mytaskbar
    {
        static bool taskbar_support = true;
        public static void set(int state, int cur, int total)
        {
            if (taskbar_support)
            {
                try
                {
                    set_internal(state, cur, total);
                }
                catch (Exception e)
                {
                    mylog.log("Error: " + e.Message);
                    taskbar_support = false;
                }
            }
        }
        private static void set_internal(int state, int cur, int total)
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
