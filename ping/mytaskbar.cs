using Microsoft.WindowsAPICodePack.Taskbar;
using System;

namespace rabbit
{
    class mytaskbar
    {
        bool taskbar_support = true;
        string errmsg;

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
                    errmsg = e.Message;
                    taskbar_support = false;
                }
            }
            return taskbar_support;
        }
        public string strerr()
        {
            return errmsg;
        }
        private void set_internal(int state, int cur, int total)
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
