namespace lazebird.rabbit.rabbit
{
    partial class Form1
    {
        /// <summary>
        /// 必需的设计器变量。
        /// </summary>
        System.ComponentModel.IContainer components = null;

        /// <summary>
        /// 清理所有正在使用的资源。
        /// </summary>
        /// <param name="disposing">如果应释放托管资源，为 true；否则为 false。</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                ntfico.Visible = false;
                components.Dispose();
            }
            base.Dispose(disposing);
            httpd.Dispose();
            httpdlog.Dispose();
            pinglog.Dispose();
            setlog.Dispose();
            tftpclog.Dispose();
            tftpd.Dispose();
            tftpdlog.Dispose();
            planlog.Dispose();
            chat.Dispose();
            chatlog.Dispose();
        }

        #region Windows 窗体设计器生成的代码

        /// <summary>
        /// 设计器支持所需的方法 - 不要修改
        /// 使用代码编辑器修改此方法的内容。
        /// </summary>
        void InitializeComponent()
        {
            this.components = new System.ComponentModel.Container();
            System.ComponentModel.ComponentResourceManager resources = new System.ComponentModel.ComponentResourceManager(typeof(Form1));
            this.toolStripContainer1 = new System.Windows.Forms.ToolStripContainer();
            this.tabs = new System.Windows.Forms.TabControl();
            this.Ping = new System.Windows.Forms.TabPage();
            this.label_addr = new System.Windows.Forms.Label();
            this.text_addr = new System.Windows.Forms.TextBox();
            this.label_interval = new System.Windows.Forms.Label();
            this.text_interval = new System.Windows.Forms.TextBox();
            this.label_count = new System.Windows.Forms.Label();
            this.text_count = new System.Windows.Forms.TextBox();
            this.btn_ping_log = new System.Windows.Forms.Button();
            this.text_logpath = new System.Windows.Forms.TextBox();
            this.btn_ping = new System.Windows.Forms.Button();
            this.ping_output = new System.Windows.Forms.ListBox();
            this.Scan = new System.Windows.Forms.TabPage();
            this.label_scanip = new System.Windows.Forms.Label();
            this.text_scanstart = new System.Windows.Forms.TextBox();
            this.label_scanend = new System.Windows.Forms.Label();
            this.text_scanend = new System.Windows.Forms.TextBox();
            this.btn_scan = new System.Windows.Forms.Button();
            this.fp_scan = new System.Windows.Forms.FlowLayoutPanel();
            this.HTTPD = new System.Windows.Forms.TabPage();
            this.label_http_port = new System.Windows.Forms.Label();
            this.text_http_port = new System.Windows.Forms.TextBox();
            this.cb_http_shell = new System.Windows.Forms.CheckBox();
            this.cb_http_index = new System.Windows.Forms.CheckBox();
            this.btn_httpd = new System.Windows.Forms.Button();
            this.fp_httpd = new System.Windows.Forms.FlowLayoutPanel();
            this.httpd_output = new System.Windows.Forms.ListBox();
            this.TFTPD = new System.Windows.Forms.TabPage();
            this.label_tftpdtout = new System.Windows.Forms.Label();
            this.text_tftpdtout = new System.Windows.Forms.TextBox();
            this.label_tftpdretry = new System.Windows.Forms.Label();
            this.text_tftpdretry = new System.Windows.Forms.TextBox();
            this.tftpd_fp = new System.Windows.Forms.FlowLayoutPanel();
            this.tftpd_adddir = new System.Windows.Forms.Button();
            this.tftpd_deldir = new System.Windows.Forms.Button();
            this.tftpd_btn = new System.Windows.Forms.Button();
            this.tftpd_output = new System.Windows.Forms.ListBox();
            this.TFTPC = new System.Windows.Forms.TabPage();
            this.label_tftpcaddr = new System.Windows.Forms.Label();
            this.text_tftpcaddr = new System.Windows.Forms.TextBox();
            this.label_tftpctout = new System.Windows.Forms.Label();
            this.text_tftpctout = new System.Windows.Forms.TextBox();
            this.label_tftpcretry = new System.Windows.Forms.Label();
            this.text_tftpcretry = new System.Windows.Forms.TextBox();
            this.label_tftpcblksize = new System.Windows.Forms.Label();
            this.text_tftpcblksize = new System.Windows.Forms.TextBox();
            this.btn_tftpclfile = new System.Windows.Forms.Button();
            this.text_tftpclfile = new System.Windows.Forms.TextBox();
            this.label_tftpcrfile = new System.Windows.Forms.Label();
            this.text_tftpcrfile = new System.Windows.Forms.TextBox();
            this.btn_tftpcget = new System.Windows.Forms.Button();
            this.btn_tftpcput = new System.Windows.Forms.Button();
            this.tftpc_output = new System.Windows.Forms.ListBox();
            this.PLAN = new System.Windows.Forms.TabPage();
            this.fp_plan = new System.Windows.Forms.FlowLayoutPanel();
            this.dt1_plan = new System.Windows.Forms.DateTimePicker();
            this.dt2_plan = new System.Windows.Forms.DateTimePicker();
            this.label_plancycle = new System.Windows.Forms.Label();
            this.text_plancycle = new System.Windows.Forms.TextBox();
            this.cb_planunit = new System.Windows.Forms.ComboBox();
            this.text_planmsg = new System.Windows.Forms.TextBox();
            this.btn_plandel = new System.Windows.Forms.Button();
            this.btn_planadd = new System.Windows.Forms.Button();
            this.plan_output = new System.Windows.Forms.ListBox();
            this.CHAT = new System.Windows.Forms.TabPage();
            this.text_chatntf = new System.Windows.Forms.TextBox();
            this.text_chatname = new System.Windows.Forms.TextBox();
            this.btn_chat = new System.Windows.Forms.Button();
            this.btn_chatrefresh = new System.Windows.Forms.Button();
            this.btn_chatntf = new System.Windows.Forms.Button();
            this.fp_chat = new System.Windows.Forms.FlowLayoutPanel();
            this.chat_output = new System.Windows.Forms.ListBox();
            this.Setting = new System.Windows.Forms.TabPage();
            this.label_lang = new System.Windows.Forms.Label();
            this.lang_cb = new System.Windows.Forms.ComboBox();
            this.label_ver = new System.Windows.Forms.Label();
            this.link_ver = new System.Windows.Forms.LinkLabel();
            this.label_prj = new System.Windows.Forms.Label();
            this.link_prj = new System.Windows.Forms.LinkLabel();
            this.label_prof = new System.Windows.Forms.Label();
            this.link_prof = new System.Windows.Forms.LinkLabel();
            this.cb_systray = new System.Windows.Forms.CheckBox();
            this.setting_output = new System.Windows.Forms.ListBox();
            this.ntfico = new System.Windows.Forms.NotifyIcon(this.components);
            this.toolStripContainer1.ContentPanel.SuspendLayout();
            this.toolStripContainer1.SuspendLayout();
            this.tabs.SuspendLayout();
            this.Ping.SuspendLayout();
            this.Scan.SuspendLayout();
            this.HTTPD.SuspendLayout();
            this.TFTPD.SuspendLayout();
            this.TFTPC.SuspendLayout();
            this.PLAN.SuspendLayout();
            this.CHAT.SuspendLayout();
            this.Setting.SuspendLayout();
            this.SuspendLayout();
            // 
            // toolStripContainer1
            // 
            this.toolStripContainer1.BottomToolStripPanelVisible = false;
            // 
            // toolStripContainer1.ContentPanel
            // 
            resources.ApplyResources(this.toolStripContainer1.ContentPanel, "toolStripContainer1.ContentPanel");
            this.toolStripContainer1.ContentPanel.Controls.Add(this.tabs);
            resources.ApplyResources(this.toolStripContainer1, "toolStripContainer1");
            this.toolStripContainer1.LeftToolStripPanelVisible = false;
            this.toolStripContainer1.Name = "toolStripContainer1";
            this.toolStripContainer1.RightToolStripPanelVisible = false;
            this.toolStripContainer1.TopToolStripPanelVisible = false;
            // 
            // tabs
            // 
            this.tabs.Controls.Add(this.Ping);
            this.tabs.Controls.Add(this.Scan);
            this.tabs.Controls.Add(this.HTTPD);
            this.tabs.Controls.Add(this.TFTPD);
            this.tabs.Controls.Add(this.TFTPC);
            this.tabs.Controls.Add(this.PLAN);
            this.tabs.Controls.Add(this.CHAT);
            this.tabs.Controls.Add(this.Setting);
            resources.ApplyResources(this.tabs, "tabs");
            this.tabs.Multiline = true;
            this.tabs.Name = "tabs";
            this.tabs.SelectedIndex = 0;
            // 
            // Ping
            // 
            this.Ping.BackColor = System.Drawing.Color.DimGray;
            this.Ping.Controls.Add(this.label_addr);
            this.Ping.Controls.Add(this.text_addr);
            this.Ping.Controls.Add(this.label_interval);
            this.Ping.Controls.Add(this.text_interval);
            this.Ping.Controls.Add(this.label_count);
            this.Ping.Controls.Add(this.text_count);
            this.Ping.Controls.Add(this.btn_ping_log);
            this.Ping.Controls.Add(this.text_logpath);
            this.Ping.Controls.Add(this.btn_ping);
            this.Ping.Controls.Add(this.ping_output);
            this.Ping.ForeColor = System.Drawing.Color.White;
            resources.ApplyResources(this.Ping, "Ping");
            this.Ping.Name = "Ping";
            // 
            // label_addr
            // 
            resources.ApplyResources(this.label_addr, "label_addr");
            this.label_addr.BackColor = System.Drawing.Color.Transparent;
            this.label_addr.Name = "label_addr";
            // 
            // text_addr
            // 
            resources.ApplyResources(this.text_addr, "text_addr");
            this.text_addr.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_addr.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_addr.ForeColor = System.Drawing.Color.White;
            this.text_addr.Name = "text_addr";
            // 
            // label_interval
            // 
            resources.ApplyResources(this.label_interval, "label_interval");
            this.label_interval.BackColor = System.Drawing.Color.Transparent;
            this.label_interval.Name = "label_interval";
            // 
            // text_interval
            // 
            resources.ApplyResources(this.text_interval, "text_interval");
            this.text_interval.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_interval.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_interval.ForeColor = System.Drawing.Color.White;
            this.text_interval.Name = "text_interval";
            // 
            // label_count
            // 
            resources.ApplyResources(this.label_count, "label_count");
            this.label_count.BackColor = System.Drawing.Color.Transparent;
            this.label_count.Name = "label_count";
            // 
            // text_count
            // 
            resources.ApplyResources(this.text_count, "text_count");
            this.text_count.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_count.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_count.ForeColor = System.Drawing.Color.White;
            this.text_count.Name = "text_count";
            // 
            // btn_ping_log
            // 
            this.btn_ping_log.BackColor = System.Drawing.Color.YellowGreen;
            resources.ApplyResources(this.btn_ping_log, "btn_ping_log");
            this.btn_ping_log.FlatAppearance.BorderColor = System.Drawing.Color.FromArgb(((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))));
            this.btn_ping_log.FlatAppearance.BorderSize = 0;
            this.btn_ping_log.FlatAppearance.MouseDownBackColor = System.Drawing.Color.Transparent;
            this.btn_ping_log.FlatAppearance.MouseOverBackColor = System.Drawing.Color.Transparent;
            this.btn_ping_log.ForeColor = System.Drawing.Color.White;
            this.btn_ping_log.Name = "btn_ping_log";
            this.btn_ping_log.UseVisualStyleBackColor = false;
            // 
            // text_logpath
            // 
            resources.ApplyResources(this.text_logpath, "text_logpath");
            this.text_logpath.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_logpath.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_logpath.ForeColor = System.Drawing.Color.White;
            this.text_logpath.Name = "text_logpath";
            // 
            // btn_ping
            // 
            resources.ApplyResources(this.btn_ping, "btn_ping");
            this.btn_ping.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_ping.FlatAppearance.BorderColor = System.Drawing.Color.FromArgb(((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))));
            this.btn_ping.FlatAppearance.BorderSize = 0;
            this.btn_ping.FlatAppearance.MouseDownBackColor = System.Drawing.Color.Transparent;
            this.btn_ping.FlatAppearance.MouseOverBackColor = System.Drawing.Color.Transparent;
            this.btn_ping.ForeColor = System.Drawing.Color.White;
            this.btn_ping.Name = "btn_ping";
            this.btn_ping.UseVisualStyleBackColor = false;
            // 
            // ping_output
            // 
            resources.ApplyResources(this.ping_output, "ping_output");
            this.ping_output.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.ping_output.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.ping_output.ForeColor = System.Drawing.Color.White;
            this.ping_output.FormattingEnabled = true;
            this.ping_output.Name = "ping_output";
            // 
            // Scan
            // 
            this.Scan.BackColor = System.Drawing.Color.DimGray;
            this.Scan.Controls.Add(this.label_scanip);
            this.Scan.Controls.Add(this.text_scanstart);
            this.Scan.Controls.Add(this.label_scanend);
            this.Scan.Controls.Add(this.text_scanend);
            this.Scan.Controls.Add(this.btn_scan);
            this.Scan.Controls.Add(this.fp_scan);
            this.Scan.ForeColor = System.Drawing.Color.White;
            resources.ApplyResources(this.Scan, "Scan");
            this.Scan.Name = "Scan";
            // 
            // label_scanip
            // 
            resources.ApplyResources(this.label_scanip, "label_scanip");
            this.label_scanip.BackColor = System.Drawing.Color.Transparent;
            this.label_scanip.Name = "label_scanip";
            // 
            // text_scanstart
            // 
            resources.ApplyResources(this.text_scanstart, "text_scanstart");
            this.text_scanstart.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_scanstart.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_scanstart.ForeColor = System.Drawing.Color.White;
            this.text_scanstart.Name = "text_scanstart";
            // 
            // label_scanend
            // 
            resources.ApplyResources(this.label_scanend, "label_scanend");
            this.label_scanend.BackColor = System.Drawing.Color.Transparent;
            this.label_scanend.Name = "label_scanend";
            // 
            // text_scanend
            // 
            resources.ApplyResources(this.text_scanend, "text_scanend");
            this.text_scanend.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_scanend.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_scanend.ForeColor = System.Drawing.Color.White;
            this.text_scanend.Name = "text_scanend";
            // 
            // btn_scan
            // 
            resources.ApplyResources(this.btn_scan, "btn_scan");
            this.btn_scan.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_scan.FlatAppearance.BorderSize = 0;
            this.btn_scan.Name = "btn_scan";
            this.btn_scan.UseVisualStyleBackColor = false;
            // 
            // fp_scan
            // 
            resources.ApplyResources(this.fp_scan, "fp_scan");
            this.fp_scan.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.fp_scan.ForeColor = System.Drawing.Color.White;
            this.fp_scan.Name = "fp_scan";
            // 
            // HTTPD
            // 
            this.HTTPD.BackColor = System.Drawing.Color.DimGray;
            this.HTTPD.Controls.Add(this.label_http_port);
            this.HTTPD.Controls.Add(this.text_http_port);
            this.HTTPD.Controls.Add(this.cb_http_shell);
            this.HTTPD.Controls.Add(this.cb_http_index);
            this.HTTPD.Controls.Add(this.btn_httpd);
            this.HTTPD.Controls.Add(this.fp_httpd);
            this.HTTPD.Controls.Add(this.httpd_output);
            this.HTTPD.ForeColor = System.Drawing.Color.White;
            resources.ApplyResources(this.HTTPD, "HTTPD");
            this.HTTPD.Name = "HTTPD";
            // 
            // label_http_port
            // 
            resources.ApplyResources(this.label_http_port, "label_http_port");
            this.label_http_port.Name = "label_http_port";
            // 
            // text_http_port
            // 
            resources.ApplyResources(this.text_http_port, "text_http_port");
            this.text_http_port.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_http_port.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_http_port.ForeColor = System.Drawing.Color.White;
            this.text_http_port.Name = "text_http_port";
            // 
            // cb_http_shell
            // 
            resources.ApplyResources(this.cb_http_shell, "cb_http_shell");
            this.cb_http_shell.Name = "cb_http_shell";
            this.cb_http_shell.UseVisualStyleBackColor = true;
            // 
            // cb_http_index
            // 
            resources.ApplyResources(this.cb_http_index, "cb_http_index");
            this.cb_http_index.Name = "cb_http_index";
            this.cb_http_index.UseVisualStyleBackColor = true;
            // 
            // btn_httpd
            // 
            resources.ApplyResources(this.btn_httpd, "btn_httpd");
            this.btn_httpd.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_httpd.FlatAppearance.BorderSize = 0;
            this.btn_httpd.Name = "btn_httpd";
            this.btn_httpd.UseVisualStyleBackColor = false;
            // 
            // fp_httpd
            // 
            resources.ApplyResources(this.fp_httpd, "fp_httpd");
            this.fp_httpd.Name = "fp_httpd";
            // 
            // httpd_output
            // 
            resources.ApplyResources(this.httpd_output, "httpd_output");
            this.httpd_output.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.httpd_output.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.httpd_output.ForeColor = System.Drawing.Color.White;
            this.httpd_output.FormattingEnabled = true;
            this.httpd_output.Name = "httpd_output";
            // 
            // TFTPD
            // 
            this.TFTPD.BackColor = System.Drawing.Color.DimGray;
            this.TFTPD.Controls.Add(this.label_tftpdtout);
            this.TFTPD.Controls.Add(this.text_tftpdtout);
            this.TFTPD.Controls.Add(this.label_tftpdretry);
            this.TFTPD.Controls.Add(this.text_tftpdretry);
            this.TFTPD.Controls.Add(this.tftpd_fp);
            this.TFTPD.Controls.Add(this.tftpd_adddir);
            this.TFTPD.Controls.Add(this.tftpd_deldir);
            this.TFTPD.Controls.Add(this.tftpd_btn);
            this.TFTPD.Controls.Add(this.tftpd_output);
            this.TFTPD.ForeColor = System.Drawing.Color.White;
            resources.ApplyResources(this.TFTPD, "TFTPD");
            this.TFTPD.Name = "TFTPD";
            // 
            // label_tftpdtout
            // 
            resources.ApplyResources(this.label_tftpdtout, "label_tftpdtout");
            this.label_tftpdtout.Name = "label_tftpdtout";
            // 
            // text_tftpdtout
            // 
            resources.ApplyResources(this.text_tftpdtout, "text_tftpdtout");
            this.text_tftpdtout.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_tftpdtout.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_tftpdtout.ForeColor = System.Drawing.Color.White;
            this.text_tftpdtout.Name = "text_tftpdtout";
            // 
            // label_tftpdretry
            // 
            resources.ApplyResources(this.label_tftpdretry, "label_tftpdretry");
            this.label_tftpdretry.Name = "label_tftpdretry";
            // 
            // text_tftpdretry
            // 
            resources.ApplyResources(this.text_tftpdretry, "text_tftpdretry");
            this.text_tftpdretry.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_tftpdretry.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_tftpdretry.ForeColor = System.Drawing.Color.White;
            this.text_tftpdretry.Name = "text_tftpdretry";
            // 
            // tftpd_fp
            // 
            resources.ApplyResources(this.tftpd_fp, "tftpd_fp");
            this.tftpd_fp.Name = "tftpd_fp";
            // 
            // tftpd_adddir
            // 
            this.tftpd_adddir.BackColor = System.Drawing.Color.YellowGreen;
            this.tftpd_adddir.FlatAppearance.BorderSize = 0;
            resources.ApplyResources(this.tftpd_adddir, "tftpd_adddir");
            this.tftpd_adddir.Name = "tftpd_adddir";
            this.tftpd_adddir.UseVisualStyleBackColor = false;
            // 
            // tftpd_deldir
            // 
            this.tftpd_deldir.BackColor = System.Drawing.Color.YellowGreen;
            this.tftpd_deldir.FlatAppearance.BorderSize = 0;
            resources.ApplyResources(this.tftpd_deldir, "tftpd_deldir");
            this.tftpd_deldir.Name = "tftpd_deldir";
            this.tftpd_deldir.UseVisualStyleBackColor = false;
            // 
            // tftpd_btn
            // 
            resources.ApplyResources(this.tftpd_btn, "tftpd_btn");
            this.tftpd_btn.BackColor = System.Drawing.Color.YellowGreen;
            this.tftpd_btn.FlatAppearance.BorderSize = 0;
            this.tftpd_btn.Name = "tftpd_btn";
            this.tftpd_btn.UseVisualStyleBackColor = false;
            // 
            // tftpd_output
            // 
            resources.ApplyResources(this.tftpd_output, "tftpd_output");
            this.tftpd_output.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.tftpd_output.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.tftpd_output.ForeColor = System.Drawing.Color.White;
            this.tftpd_output.FormattingEnabled = true;
            this.tftpd_output.Name = "tftpd_output";
            // 
            // TFTPC
            // 
            this.TFTPC.BackColor = System.Drawing.Color.DimGray;
            this.TFTPC.Controls.Add(this.label_tftpcaddr);
            this.TFTPC.Controls.Add(this.text_tftpcaddr);
            this.TFTPC.Controls.Add(this.label_tftpctout);
            this.TFTPC.Controls.Add(this.text_tftpctout);
            this.TFTPC.Controls.Add(this.label_tftpcretry);
            this.TFTPC.Controls.Add(this.text_tftpcretry);
            this.TFTPC.Controls.Add(this.label_tftpcblksize);
            this.TFTPC.Controls.Add(this.text_tftpcblksize);
            this.TFTPC.Controls.Add(this.btn_tftpclfile);
            this.TFTPC.Controls.Add(this.text_tftpclfile);
            this.TFTPC.Controls.Add(this.label_tftpcrfile);
            this.TFTPC.Controls.Add(this.text_tftpcrfile);
            this.TFTPC.Controls.Add(this.btn_tftpcget);
            this.TFTPC.Controls.Add(this.btn_tftpcput);
            this.TFTPC.Controls.Add(this.tftpc_output);
            this.TFTPC.ForeColor = System.Drawing.Color.White;
            resources.ApplyResources(this.TFTPC, "TFTPC");
            this.TFTPC.Name = "TFTPC";
            // 
            // label_tftpcaddr
            // 
            resources.ApplyResources(this.label_tftpcaddr, "label_tftpcaddr");
            this.label_tftpcaddr.Name = "label_tftpcaddr";
            // 
            // text_tftpcaddr
            // 
            resources.ApplyResources(this.text_tftpcaddr, "text_tftpcaddr");
            this.text_tftpcaddr.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_tftpcaddr.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_tftpcaddr.ForeColor = System.Drawing.Color.White;
            this.text_tftpcaddr.Name = "text_tftpcaddr";
            // 
            // label_tftpctout
            // 
            resources.ApplyResources(this.label_tftpctout, "label_tftpctout");
            this.label_tftpctout.Name = "label_tftpctout";
            // 
            // text_tftpctout
            // 
            resources.ApplyResources(this.text_tftpctout, "text_tftpctout");
            this.text_tftpctout.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_tftpctout.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_tftpctout.ForeColor = System.Drawing.Color.White;
            this.text_tftpctout.Name = "text_tftpctout";
            // 
            // label_tftpcretry
            // 
            resources.ApplyResources(this.label_tftpcretry, "label_tftpcretry");
            this.label_tftpcretry.Name = "label_tftpcretry";
            // 
            // text_tftpcretry
            // 
            resources.ApplyResources(this.text_tftpcretry, "text_tftpcretry");
            this.text_tftpcretry.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_tftpcretry.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_tftpcretry.ForeColor = System.Drawing.Color.White;
            this.text_tftpcretry.Name = "text_tftpcretry";
            // 
            // label_tftpcblksize
            // 
            resources.ApplyResources(this.label_tftpcblksize, "label_tftpcblksize");
            this.label_tftpcblksize.Name = "label_tftpcblksize";
            // 
            // text_tftpcblksize
            // 
            resources.ApplyResources(this.text_tftpcblksize, "text_tftpcblksize");
            this.text_tftpcblksize.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_tftpcblksize.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_tftpcblksize.ForeColor = System.Drawing.Color.White;
            this.text_tftpcblksize.Name = "text_tftpcblksize";
            // 
            // btn_tftpclfile
            // 
            this.btn_tftpclfile.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_tftpclfile.FlatAppearance.BorderSize = 0;
            resources.ApplyResources(this.btn_tftpclfile, "btn_tftpclfile");
            this.btn_tftpclfile.Name = "btn_tftpclfile";
            this.btn_tftpclfile.UseVisualStyleBackColor = false;
            // 
            // text_tftpclfile
            // 
            resources.ApplyResources(this.text_tftpclfile, "text_tftpclfile");
            this.text_tftpclfile.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_tftpclfile.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_tftpclfile.ForeColor = System.Drawing.Color.White;
            this.text_tftpclfile.Name = "text_tftpclfile";
            // 
            // label_tftpcrfile
            // 
            resources.ApplyResources(this.label_tftpcrfile, "label_tftpcrfile");
            this.label_tftpcrfile.Name = "label_tftpcrfile";
            // 
            // text_tftpcrfile
            // 
            resources.ApplyResources(this.text_tftpcrfile, "text_tftpcrfile");
            this.text_tftpcrfile.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.text_tftpcrfile.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.text_tftpcrfile.ForeColor = System.Drawing.Color.White;
            this.text_tftpcrfile.Name = "text_tftpcrfile";
            // 
            // btn_tftpcget
            // 
            resources.ApplyResources(this.btn_tftpcget, "btn_tftpcget");
            this.btn_tftpcget.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_tftpcget.FlatAppearance.BorderSize = 0;
            this.btn_tftpcget.Name = "btn_tftpcget";
            this.btn_tftpcget.UseVisualStyleBackColor = false;
            // 
            // btn_tftpcput
            // 
            resources.ApplyResources(this.btn_tftpcput, "btn_tftpcput");
            this.btn_tftpcput.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_tftpcput.FlatAppearance.BorderSize = 0;
            this.btn_tftpcput.Name = "btn_tftpcput";
            this.btn_tftpcput.UseVisualStyleBackColor = false;
            // 
            // tftpc_output
            // 
            resources.ApplyResources(this.tftpc_output, "tftpc_output");
            this.tftpc_output.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.tftpc_output.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.tftpc_output.ForeColor = System.Drawing.Color.White;
            this.tftpc_output.FormattingEnabled = true;
            this.tftpc_output.Name = "tftpc_output";
            // 
            // PLAN
            // 
            this.PLAN.BackColor = System.Drawing.Color.DimGray;
            this.PLAN.Controls.Add(this.fp_plan);
            this.PLAN.Controls.Add(this.dt1_plan);
            this.PLAN.Controls.Add(this.dt2_plan);
            this.PLAN.Controls.Add(this.label_plancycle);
            this.PLAN.Controls.Add(this.text_plancycle);
            this.PLAN.Controls.Add(this.cb_planunit);
            this.PLAN.Controls.Add(this.text_planmsg);
            this.PLAN.Controls.Add(this.btn_plandel);
            this.PLAN.Controls.Add(this.btn_planadd);
            this.PLAN.Controls.Add(this.plan_output);
            this.PLAN.ForeColor = System.Drawing.Color.White;
            resources.ApplyResources(this.PLAN, "PLAN");
            this.PLAN.Name = "PLAN";
            // 
            // fp_plan
            // 
            resources.ApplyResources(this.fp_plan, "fp_plan");
            this.fp_plan.Name = "fp_plan";
            // 
            // dt1_plan
            // 
            resources.ApplyResources(this.dt1_plan, "dt1_plan");
            this.dt1_plan.Name = "dt1_plan";
            // 
            // dt2_plan
            // 
            resources.ApplyResources(this.dt2_plan, "dt2_plan");
            this.dt2_plan.Format = System.Windows.Forms.DateTimePickerFormat.Custom;
            this.dt2_plan.Name = "dt2_plan";
            this.dt2_plan.ShowUpDown = true;
            // 
            // label_plancycle
            // 
            resources.ApplyResources(this.label_plancycle, "label_plancycle");
            this.label_plancycle.Name = "label_plancycle";
            // 
            // text_plancycle
            // 
            resources.ApplyResources(this.text_plancycle, "text_plancycle");
            this.text_plancycle.Name = "text_plancycle";
            // 
            // cb_planunit
            // 
            resources.ApplyResources(this.cb_planunit, "cb_planunit");
            this.cb_planunit.FormattingEnabled = true;
            this.cb_planunit.Name = "cb_planunit";
            // 
            // text_planmsg
            // 
            resources.ApplyResources(this.text_planmsg, "text_planmsg");
            this.text_planmsg.Name = "text_planmsg";
            // 
            // btn_plandel
            // 
            resources.ApplyResources(this.btn_plandel, "btn_plandel");
            this.btn_plandel.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_plandel.FlatAppearance.BorderSize = 0;
            this.btn_plandel.Name = "btn_plandel";
            this.btn_plandel.UseVisualStyleBackColor = false;
            // 
            // btn_planadd
            // 
            resources.ApplyResources(this.btn_planadd, "btn_planadd");
            this.btn_planadd.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_planadd.FlatAppearance.BorderSize = 0;
            this.btn_planadd.Name = "btn_planadd";
            this.btn_planadd.UseVisualStyleBackColor = false;
            // 
            // plan_output
            // 
            resources.ApplyResources(this.plan_output, "plan_output");
            this.plan_output.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.plan_output.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.plan_output.ForeColor = System.Drawing.Color.White;
            this.plan_output.FormattingEnabled = true;
            this.plan_output.Name = "plan_output";
            // 
            // CHAT
            // 
            this.CHAT.BackColor = System.Drawing.Color.DimGray;
            this.CHAT.Controls.Add(this.text_chatntf);
            this.CHAT.Controls.Add(this.text_chatname);
            this.CHAT.Controls.Add(this.btn_chat);
            this.CHAT.Controls.Add(this.btn_chatrefresh);
            this.CHAT.Controls.Add(this.btn_chatntf);
            this.CHAT.Controls.Add(this.fp_chat);
            this.CHAT.Controls.Add(this.chat_output);
            this.CHAT.ForeColor = System.Drawing.Color.White;
            resources.ApplyResources(this.CHAT, "CHAT");
            this.CHAT.Name = "CHAT";
            // 
            // text_chatntf
            // 
            resources.ApplyResources(this.text_chatntf, "text_chatntf");
            this.text_chatntf.Name = "text_chatntf";
            // 
            // text_chatname
            // 
            resources.ApplyResources(this.text_chatname, "text_chatname");
            this.text_chatname.Name = "text_chatname";
            // 
            // btn_chat
            // 
            resources.ApplyResources(this.btn_chat, "btn_chat");
            this.btn_chat.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_chat.FlatAppearance.BorderSize = 0;
            this.btn_chat.Name = "btn_chat";
            this.btn_chat.UseVisualStyleBackColor = false;
            // 
            // btn_chatrefresh
            // 
            resources.ApplyResources(this.btn_chatrefresh, "btn_chatrefresh");
            this.btn_chatrefresh.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_chatrefresh.FlatAppearance.BorderSize = 0;
            this.btn_chatrefresh.Name = "btn_chatrefresh";
            this.btn_chatrefresh.UseVisualStyleBackColor = false;
            // 
            // btn_chatntf
            // 
            resources.ApplyResources(this.btn_chatntf, "btn_chatntf");
            this.btn_chatntf.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_chatntf.FlatAppearance.BorderSize = 0;
            this.btn_chatntf.Name = "btn_chatntf";
            this.btn_chatntf.UseVisualStyleBackColor = false;
            // 
            // fp_chat
            // 
            resources.ApplyResources(this.fp_chat, "fp_chat");
            this.fp_chat.Name = "fp_chat";
            // 
            // chat_output
            // 
            resources.ApplyResources(this.chat_output, "chat_output");
            this.chat_output.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.chat_output.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.chat_output.ForeColor = System.Drawing.Color.White;
            this.chat_output.FormattingEnabled = true;
            this.chat_output.Name = "chat_output";
            // 
            // Setting
            // 
            this.Setting.BackColor = System.Drawing.Color.DimGray;
            this.Setting.Controls.Add(this.label_lang);
            this.Setting.Controls.Add(this.lang_cb);
            this.Setting.Controls.Add(this.label_ver);
            this.Setting.Controls.Add(this.link_ver);
            this.Setting.Controls.Add(this.label_prj);
            this.Setting.Controls.Add(this.link_prj);
            this.Setting.Controls.Add(this.label_prof);
            this.Setting.Controls.Add(this.link_prof);
            this.Setting.Controls.Add(this.cb_systray);
            this.Setting.Controls.Add(this.setting_output);
            this.Setting.ForeColor = System.Drawing.Color.White;
            resources.ApplyResources(this.Setting, "Setting");
            this.Setting.Name = "Setting";
            // 
            // label_lang
            // 
            resources.ApplyResources(this.label_lang, "label_lang");
            this.label_lang.Name = "label_lang";
            // 
            // lang_cb
            // 
            this.lang_cb.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            resources.ApplyResources(this.lang_cb, "lang_cb");
            this.lang_cb.ForeColor = System.Drawing.Color.White;
            this.lang_cb.FormattingEnabled = true;
            this.lang_cb.Name = "lang_cb";
            // 
            // label_ver
            // 
            resources.ApplyResources(this.label_ver, "label_ver");
            this.label_ver.Name = "label_ver";
            // 
            // link_ver
            // 
            resources.ApplyResources(this.link_ver, "link_ver");
            this.link_ver.Name = "link_ver";
            // 
            // label_prj
            // 
            resources.ApplyResources(this.label_prj, "label_prj");
            this.label_prj.Name = "label_prj";
            // 
            // link_prj
            // 
            resources.ApplyResources(this.link_prj, "link_prj");
            this.link_prj.Name = "link_prj";
            // 
            // label_prof
            // 
            resources.ApplyResources(this.label_prof, "label_prof");
            this.label_prof.Name = "label_prof";
            // 
            // link_prof
            // 
            resources.ApplyResources(this.link_prof, "link_prof");
            this.link_prof.Name = "link_prof";
            // 
            // cb_systray
            // 
            resources.ApplyResources(this.cb_systray, "cb_systray");
            this.cb_systray.Name = "cb_systray";
            this.cb_systray.UseVisualStyleBackColor = true;
            // 
            // setting_output
            // 
            resources.ApplyResources(this.setting_output, "setting_output");
            this.setting_output.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(64)))), ((int)(((byte)(64)))), ((int)(((byte)(64)))));
            this.setting_output.BorderStyle = System.Windows.Forms.BorderStyle.None;
            this.setting_output.ForeColor = System.Drawing.Color.White;
            this.setting_output.FormattingEnabled = true;
            this.setting_output.Name = "setting_output";
            // 
            // ntfico
            // 
            resources.ApplyResources(this.ntfico, "ntfico");
            // 
            // Form1
            // 
            resources.ApplyResources(this, "$this");
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.BackColor = System.Drawing.SystemColors.ButtonFace;
            this.Controls.Add(this.toolStripContainer1);
            this.DoubleBuffered = true;
            this.ForeColor = System.Drawing.Color.White;
            this.Name = "Form1";
            this.toolStripContainer1.ContentPanel.ResumeLayout(false);
            this.toolStripContainer1.ResumeLayout(false);
            this.toolStripContainer1.PerformLayout();
            this.tabs.ResumeLayout(false);
            this.Ping.ResumeLayout(false);
            this.Ping.PerformLayout();
            this.Scan.ResumeLayout(false);
            this.Scan.PerformLayout();
            this.HTTPD.ResumeLayout(false);
            this.HTTPD.PerformLayout();
            this.TFTPD.ResumeLayout(false);
            this.TFTPD.PerformLayout();
            this.TFTPC.ResumeLayout(false);
            this.TFTPC.PerformLayout();
            this.PLAN.ResumeLayout(false);
            this.PLAN.PerformLayout();
            this.CHAT.ResumeLayout(false);
            this.CHAT.PerformLayout();
            this.Setting.ResumeLayout(false);
            this.Setting.PerformLayout();
            this.ResumeLayout(false);

        }

        #endregion

        System.Windows.Forms.Label label_interval;
        System.Windows.Forms.Label label_count;
        System.Windows.Forms.TextBox text_addr;
        System.Windows.Forms.TextBox text_interval;
        System.Windows.Forms.ToolStripContainer toolStripContainer1;
        System.Windows.Forms.Button btn_ping;
        System.Windows.Forms.Button btn_ping_log;
        System.Windows.Forms.TextBox text_count;
        System.Windows.Forms.Label label_addr;
        System.Windows.Forms.TextBox text_logpath;
        System.Windows.Forms.TabControl tabs;
        System.Windows.Forms.TabPage Ping;
        System.Windows.Forms.TabPage HTTPD;
        System.Windows.Forms.ListBox ping_output;
        System.Windows.Forms.TabPage PLAN;
        System.Windows.Forms.TabPage Setting;
        System.Windows.Forms.Label label_http_port;
        System.Windows.Forms.TextBox text_http_port;
        System.Windows.Forms.Button btn_httpd;
        System.Windows.Forms.TabPage TFTPD;
        System.Windows.Forms.Button tftpd_adddir;
        System.Windows.Forms.Button tftpd_deldir;
        System.Windows.Forms.Label label_lang;
        System.Windows.Forms.ComboBox lang_cb;
        System.Windows.Forms.ListBox httpd_output;
        System.Windows.Forms.ListBox tftpd_output;
        System.Windows.Forms.Button tftpd_btn;
        System.Windows.Forms.TabPage Scan;
        System.Windows.Forms.Label label_scanip;
        System.Windows.Forms.TextBox text_scanstart;
        System.Windows.Forms.Label label_scanend;
        System.Windows.Forms.TextBox text_scanend;
        System.Windows.Forms.Button btn_scan;
        System.Windows.Forms.FlowLayoutPanel fp_scan;
        System.Windows.Forms.ListBox setting_output;
        System.Windows.Forms.LinkLabel link_prj;
        System.Windows.Forms.Label label_prj;
        System.Windows.Forms.ListBox plan_output;
        System.Windows.Forms.Label label_tftpdtout;
        System.Windows.Forms.TextBox text_tftpdtout;
        System.Windows.Forms.Label label_tftpdretry;
        System.Windows.Forms.TextBox text_tftpdretry;
        System.Windows.Forms.TabPage TFTPC;
        System.Windows.Forms.Label label_tftpctout;
        System.Windows.Forms.TextBox text_tftpctout;
        System.Windows.Forms.Label label_tftpcretry;
        System.Windows.Forms.TextBox text_tftpcretry;
        System.Windows.Forms.TextBox text_tftpcblksize;
        System.Windows.Forms.TextBox text_tftpclfile;
        System.Windows.Forms.Button btn_tftpclfile;
        System.Windows.Forms.TextBox text_tftpcrfile;
        System.Windows.Forms.Button btn_tftpcget;
        System.Windows.Forms.ListBox tftpc_output;
        System.Windows.Forms.Label label_tftpcblksize;
        System.Windows.Forms.Label label_tftpcrfile;
        System.Windows.Forms.Button btn_tftpcput;
        System.Windows.Forms.Label label_tftpcaddr;
        System.Windows.Forms.TextBox text_tftpcaddr;
        System.Windows.Forms.FlowLayoutPanel tftpd_fp;
        private System.Windows.Forms.Label label_ver;
        private System.Windows.Forms.LinkLabel link_ver;
        private System.Windows.Forms.Label label_prof;
        private System.Windows.Forms.LinkLabel link_prof;
        private System.Windows.Forms.FlowLayoutPanel fp_httpd;
        private System.Windows.Forms.CheckBox cb_http_index;
        private System.Windows.Forms.CheckBox cb_http_shell;
        private System.Windows.Forms.Button btn_planadd;
        private System.Windows.Forms.FlowLayoutPanel fp_plan;
        private System.Windows.Forms.DateTimePicker dt1_plan;
        private System.Windows.Forms.DateTimePicker dt2_plan;
        private System.Windows.Forms.TextBox text_planmsg;
        private System.Windows.Forms.TextBox text_plancycle;
        private System.Windows.Forms.Label label_plancycle;
        private System.Windows.Forms.ComboBox cb_planunit;
        private System.Windows.Forms.Button btn_plandel;
        private System.Windows.Forms.CheckBox cb_systray;
        private System.Windows.Forms.NotifyIcon ntfico;
        private System.Windows.Forms.TabPage CHAT;
        private System.Windows.Forms.FlowLayoutPanel fp_chat;
        private System.Windows.Forms.ListBox chat_output;
        private System.Windows.Forms.Button btn_chatntf;
        private System.Windows.Forms.Button btn_chatrefresh;
        private System.Windows.Forms.Button btn_chat;
        private System.Windows.Forms.TextBox text_chatname;
        private System.Windows.Forms.TextBox text_chatntf;
    }
}

