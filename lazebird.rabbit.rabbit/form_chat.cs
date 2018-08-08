using lazebird.rabbit.chat;
using lazebird.rabbit.common;
using System;
using System.Collections;
using System.Drawing;
using System.Net;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rlog chatlog;
        rchat chat;
        Hashtable chatephash;
        Hashtable chatbtnhash;
        void init_form_chat()
        {
            chatephash = new Hashtable();
            chatbtnhash = new Hashtable();
            chatlog = new rlog(chat_output);
            fp_chat.AutoScroll = true;
            chat = new rchat(chat_log_func, add_user);
            btn_chat.Click += chat_click;
            btn_chatrefresh.Click += chat_refresh_click;
            btn_chatntf.Click += chat_notify_click;
            btn_chat.PerformClick();
        }
        void chat_log_func(string msg)
        {
            chatlog.write(msg);
        }
        void chat_click(object sender, EventArgs e)
        {
            chat_log_func("I: " + btn_chat.Text);
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
            saveconf();
        }
        void user_click(object sender, EventArgs e)
        {
            IPEndPoint r = (IPEndPoint)chatbtnhash[sender];
            string ruser = (string)chatephash[r];
            chat.new_chat(r, ruser);
            chat_log_func("I: chat to " + r.ToString() + " " + ruser);
        }
        void add_user(IPEndPoint ep, string user)
        {
            if (this.InvokeRequired)
            {
                this.Invoke(new MethodInvoker(delegate { add_user(ep, user); }));
                return;
            }
            chat_log_func("I: " + ep.ToString() + " " + user);
            if (chatephash.ContainsKey(ep)) chatephash.Remove(ep);
            chatephash.Add(ep, user);
            Button b = new Button();
            b.BackColor = Color.FromArgb(64, 64, 64);
            b.FlatAppearance.BorderSize = 0;
            b.FlatStyle = FlatStyle.Flat;
            b.Width = 160;
            b.Text = user;
            b.Click += user_click;
            fp_chat.Controls.Add(b);
            chatbtnhash.Add(b, ep);
        }
        void chat_refresh_click(object sender, EventArgs e)
        {
            chat.send_query(1314);
        }
        void chat_notify_click(object sender, EventArgs e)
        {
            chat.send_notification(1314);
        }
        void chat_readconf()
        {
        }
        void chat_saveconf()
        {
            if (onloading) return;
        }
    }
}
