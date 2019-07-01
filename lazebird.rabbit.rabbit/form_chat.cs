using lazebird.rabbit.chat;
using lazebird.rabbit.common;
using System;
using System.Collections;
using System.Net;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rpanel chat_upanel;
        rlog chatlog;
        rchat chat;
        Hashtable chatephash;
        Hashtable chatbtnhash;
        void init_form_chat()
        {
            chat_upanel = new rpanel(fp_chat, 160);
            chatlog = new rlog(lb_chat);
            chatephash = new Hashtable();
            chatbtnhash = new Hashtable();
            chat = new rchat(chat_log_func, add_user);
            btn_chat.Click += chat_click;
            btn_chatrefresh.Click += chat_refresh_click;
            btn_chatntf.Click += chat_notify_click;
            text_chatname.LostFocus += chat_name_change;
        }
        void chat_log_func(string msg)
        {
            chatlog.write(msg);
        }
        void chat_name_change(object sender, EventArgs e)
        {
            chat.set_name(text_chatname.Text);
        }
        void chat_click(object sender, EventArgs e)
        {
            if (btn_chat.Text == Language.trans("开始"))
            {
                btn_chat.Text = Language.trans("停止");
                chat.start(1314);
            }
            else
            {
                btn_chat.Text = Language.trans("开始");
                chat.stop();
            }
        }
        void user_click(object sender, EventArgs e)
        {
            IPEndPoint r = (IPEndPoint)chatbtnhash[sender];
            string ruser = (string)chatephash[r];
            chat.new_chat(r, ruser);
        }
        void add_user(IPEndPoint ep, string user)
        {
            if (this.InvokeRequired)
            {
                this.Invoke(new MethodInvoker(delegate { add_user(ep, user); }));
                return;
            }
            //chat_log_func("I: " + ep.ToString() + " " + user);
            if (chatephash.ContainsKey(ep)) chatephash.Remove(ep);
            chatephash.Add(ep, user);
            TextBox tb = chat_upanel.add(user, user_click, null);
            tb.TextAlign = HorizontalAlignment.Center;
            chatbtnhash.Add(tb, ep);
        }
        void chat_refresh_click(object sender, EventArgs e)
        {
            chatephash.Clear();
            chatbtnhash.Clear();
            chat_upanel.clear();
            chat.new_query(text_chatntf.Text, 1314);
        }
        void chat_notify_click(object sender, EventArgs e)
        {
            chat.new_notification(text_chatntf.Text, 1314);
        }
    }
}
