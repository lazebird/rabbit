﻿namespace lazebird.rabbit.rabbit
{
    partial class Form1
    {
        /// <summary>
        /// 必需的设计器变量。
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// 清理所有正在使用的资源。
        /// </summary>
        /// <param name="disposing">如果应释放托管资源，为 true；否则为 false。</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows 窗体设计器生成的代码

        /// <summary>
        /// 设计器支持所需的方法 - 不要修改
        /// 使用代码编辑器修改此方法的内容。
        /// </summary>
        private void InitializeComponent()
        {
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
            this.btn_http_dir = new System.Windows.Forms.Button();
            this.text_http_dir = new System.Windows.Forms.TextBox();
            this.btn_httpd = new System.Windows.Forms.Button();
            this.httpd_output = new System.Windows.Forms.ListBox();
            this.TFTPD = new System.Windows.Forms.TabPage();
            this.tftp_dirtext1 = new System.Windows.Forms.TextBox();
            this.tftp_dirbtn1 = new System.Windows.Forms.Button();
            this.tftp_dirtext2 = new System.Windows.Forms.TextBox();
            this.tftp_dirbtn2 = new System.Windows.Forms.Button();
            this.tftp_dirtext3 = new System.Windows.Forms.TextBox();
            this.tftp_dirbtn3 = new System.Windows.Forms.Button();
            this.tftp_dirtext4 = new System.Windows.Forms.TextBox();
            this.tftp_dirbtn4 = new System.Windows.Forms.Button();
            this.tftp_dirtext5 = new System.Windows.Forms.TextBox();
            this.tftp_dirbtn5 = new System.Windows.Forms.Button();
            this.tftpd_btn = new System.Windows.Forms.Button();
            this.tftpd_output = new System.Windows.Forms.ListBox();
            this.FTPD = new System.Windows.Forms.TabPage();
            this.ftp_dirtext1 = new System.Windows.Forms.TextBox();
            this.ftp_dirbtn1 = new System.Windows.Forms.Button();
            this.ftp_dirtext2 = new System.Windows.Forms.TextBox();
            this.ftp_dirbtn2 = new System.Windows.Forms.Button();
            this.ftp_dirtext3 = new System.Windows.Forms.TextBox();
            this.ftp_dirbtn3 = new System.Windows.Forms.Button();
            this.ftp_dirtext4 = new System.Windows.Forms.TextBox();
            this.ftp_dirbtn4 = new System.Windows.Forms.Button();
            this.ftp_dirtext5 = new System.Windows.Forms.TextBox();
            this.ftp_dirbtn5 = new System.Windows.Forms.Button();
            this.btn_ftp = new System.Windows.Forms.Button();
            this.DHCPD = new System.Windows.Forms.TabPage();
            this.dhcp_startlabel = new System.Windows.Forms.Label();
            this.dhcp_starttext = new System.Windows.Forms.TextBox();
            this.dhcp_endlabel = new System.Windows.Forms.Label();
            this.dhcp_endtext = new System.Windows.Forms.TextBox();
            this.dhcp_gatewaylabel = new System.Windows.Forms.Label();
            this.dhcp_gatewaytext = new System.Windows.Forms.TextBox();
            this.dhcp_logbtn = new System.Windows.Forms.Button();
            this.dhcp_logtext = new System.Windows.Forms.TextBox();
            this.dhcp_btn = new System.Windows.Forms.Button();
            this.dhcp_logmsg = new System.Windows.Forms.ListBox();
            this.Setting = new System.Windows.Forms.TabPage();
            this.lang = new System.Windows.Forms.Label();
            this.lang_opt = new System.Windows.Forms.ComboBox();
            this.setting_output = new System.Windows.Forms.ListBox();
            this.toolStripContainer1.ContentPanel.SuspendLayout();
            this.toolStripContainer1.SuspendLayout();
            this.tabs.SuspendLayout();
            this.Ping.SuspendLayout();
            this.Scan.SuspendLayout();
            this.HTTPD.SuspendLayout();
            this.TFTPD.SuspendLayout();
            this.FTPD.SuspendLayout();
            this.DHCPD.SuspendLayout();
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
            this.tabs.Controls.Add(this.FTPD);
            this.tabs.Controls.Add(this.DHCPD);
            this.tabs.Controls.Add(this.Setting);
            resources.ApplyResources(this.tabs, "tabs");
            this.tabs.Name = "tabs";
            this.tabs.SelectedIndex = 0;
            // 
            // Ping
            // 
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
            this.btn_ping_log.ForeColor = System.Drawing.SystemColors.ControlText;
            this.btn_ping_log.Name = "btn_ping_log";
            this.btn_ping_log.UseVisualStyleBackColor = false;
            // 
            // text_logpath
            // 
            resources.ApplyResources(this.text_logpath, "text_logpath");
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
            this.btn_ping.ForeColor = System.Drawing.SystemColors.ControlText;
            this.btn_ping.Name = "btn_ping";
            this.btn_ping.UseVisualStyleBackColor = false;
            // 
            // ping_output
            // 
            resources.ApplyResources(this.ping_output, "ping_output");
            this.ping_output.FormattingEnabled = true;
            this.ping_output.Name = "ping_output";
            // 
            // Scan
            // 
            this.Scan.Controls.Add(this.label_scanip);
            this.Scan.Controls.Add(this.text_scanstart);
            this.Scan.Controls.Add(this.label_scanend);
            this.Scan.Controls.Add(this.text_scanend);
            this.Scan.Controls.Add(this.btn_scan);
            this.Scan.Controls.Add(this.fp_scan);
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
            this.fp_scan.Name = "fp_scan";
            // 
            // HTTPD
            // 
            this.HTTPD.Controls.Add(this.label_http_port);
            this.HTTPD.Controls.Add(this.text_http_port);
            this.HTTPD.Controls.Add(this.btn_http_dir);
            this.HTTPD.Controls.Add(this.text_http_dir);
            this.HTTPD.Controls.Add(this.btn_httpd);
            this.HTTPD.Controls.Add(this.httpd_output);
            resources.ApplyResources(this.HTTPD, "HTTPD");
            this.HTTPD.Name = "HTTPD";
            this.HTTPD.UseVisualStyleBackColor = true;
            // 
            // label_http_port
            // 
            resources.ApplyResources(this.label_http_port, "label_http_port");
            this.label_http_port.Name = "label_http_port";
            // 
            // text_http_port
            // 
            resources.ApplyResources(this.text_http_port, "text_http_port");
            this.text_http_port.Name = "text_http_port";
            // 
            // btn_http_dir
            // 
            this.btn_http_dir.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_http_dir.FlatAppearance.BorderSize = 0;
            resources.ApplyResources(this.btn_http_dir, "btn_http_dir");
            this.btn_http_dir.Name = "btn_http_dir";
            this.btn_http_dir.UseVisualStyleBackColor = false;
            // 
            // text_http_dir
            // 
            resources.ApplyResources(this.text_http_dir, "text_http_dir");
            this.text_http_dir.Name = "text_http_dir";
            // 
            // btn_httpd
            // 
            resources.ApplyResources(this.btn_httpd, "btn_httpd");
            this.btn_httpd.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_httpd.FlatAppearance.BorderSize = 0;
            this.btn_httpd.Name = "btn_httpd";
            this.btn_httpd.UseVisualStyleBackColor = false;
            // 
            // httpd_output
            // 
            resources.ApplyResources(this.httpd_output, "httpd_output");
            this.httpd_output.FormattingEnabled = true;
            this.httpd_output.Name = "httpd_output";
            // 
            // TFTPD
            // 
            this.TFTPD.Controls.Add(this.tftp_dirtext1);
            this.TFTPD.Controls.Add(this.tftp_dirbtn1);
            this.TFTPD.Controls.Add(this.tftp_dirtext2);
            this.TFTPD.Controls.Add(this.tftp_dirbtn2);
            this.TFTPD.Controls.Add(this.tftp_dirtext3);
            this.TFTPD.Controls.Add(this.tftp_dirbtn3);
            this.TFTPD.Controls.Add(this.tftp_dirtext4);
            this.TFTPD.Controls.Add(this.tftp_dirbtn4);
            this.TFTPD.Controls.Add(this.tftp_dirtext5);
            this.TFTPD.Controls.Add(this.tftp_dirbtn5);
            this.TFTPD.Controls.Add(this.tftpd_btn);
            this.TFTPD.Controls.Add(this.tftpd_output);
            resources.ApplyResources(this.TFTPD, "TFTPD");
            this.TFTPD.Name = "TFTPD";
            this.TFTPD.UseVisualStyleBackColor = true;
            // 
            // tftp_dirtext1
            // 
            resources.ApplyResources(this.tftp_dirtext1, "tftp_dirtext1");
            this.tftp_dirtext1.Name = "tftp_dirtext1";
            // 
            // tftp_dirbtn1
            // 
            this.tftp_dirbtn1.BackColor = System.Drawing.Color.YellowGreen;
            this.tftp_dirbtn1.FlatAppearance.BorderSize = 0;
            resources.ApplyResources(this.tftp_dirbtn1, "tftp_dirbtn1");
            this.tftp_dirbtn1.Name = "tftp_dirbtn1";
            this.tftp_dirbtn1.UseVisualStyleBackColor = false;
            // 
            // tftp_dirtext2
            // 
            resources.ApplyResources(this.tftp_dirtext2, "tftp_dirtext2");
            this.tftp_dirtext2.Name = "tftp_dirtext2";
            // 
            // tftp_dirbtn2
            // 
            this.tftp_dirbtn2.BackColor = System.Drawing.Color.YellowGreen;
            this.tftp_dirbtn2.FlatAppearance.BorderSize = 0;
            resources.ApplyResources(this.tftp_dirbtn2, "tftp_dirbtn2");
            this.tftp_dirbtn2.Name = "tftp_dirbtn2";
            this.tftp_dirbtn2.UseVisualStyleBackColor = false;
            // 
            // tftp_dirtext3
            // 
            resources.ApplyResources(this.tftp_dirtext3, "tftp_dirtext3");
            this.tftp_dirtext3.Name = "tftp_dirtext3";
            // 
            // tftp_dirbtn3
            // 
            this.tftp_dirbtn3.BackColor = System.Drawing.Color.YellowGreen;
            this.tftp_dirbtn3.FlatAppearance.BorderSize = 0;
            resources.ApplyResources(this.tftp_dirbtn3, "tftp_dirbtn3");
            this.tftp_dirbtn3.Name = "tftp_dirbtn3";
            this.tftp_dirbtn3.UseVisualStyleBackColor = false;
            // 
            // tftp_dirtext4
            // 
            resources.ApplyResources(this.tftp_dirtext4, "tftp_dirtext4");
            this.tftp_dirtext4.Name = "tftp_dirtext4";
            // 
            // tftp_dirbtn4
            // 
            this.tftp_dirbtn4.BackColor = System.Drawing.Color.YellowGreen;
            this.tftp_dirbtn4.FlatAppearance.BorderSize = 0;
            resources.ApplyResources(this.tftp_dirbtn4, "tftp_dirbtn4");
            this.tftp_dirbtn4.Name = "tftp_dirbtn4";
            this.tftp_dirbtn4.UseVisualStyleBackColor = false;
            // 
            // tftp_dirtext5
            // 
            resources.ApplyResources(this.tftp_dirtext5, "tftp_dirtext5");
            this.tftp_dirtext5.Name = "tftp_dirtext5";
            // 
            // tftp_dirbtn5
            // 
            this.tftp_dirbtn5.BackColor = System.Drawing.Color.YellowGreen;
            this.tftp_dirbtn5.FlatAppearance.BorderSize = 0;
            resources.ApplyResources(this.tftp_dirbtn5, "tftp_dirbtn5");
            this.tftp_dirbtn5.Name = "tftp_dirbtn5";
            this.tftp_dirbtn5.UseVisualStyleBackColor = false;
            // 
            // tftpd_btn
            // 
            this.tftpd_btn.BackColor = System.Drawing.Color.YellowGreen;
            this.tftpd_btn.FlatAppearance.BorderSize = 0;
            resources.ApplyResources(this.tftpd_btn, "tftpd_btn");
            this.tftpd_btn.Name = "tftpd_btn";
            this.tftpd_btn.UseVisualStyleBackColor = false;
            // 
            // tftpd_output
            // 
            resources.ApplyResources(this.tftpd_output, "tftpd_output");
            this.tftpd_output.FormattingEnabled = true;
            this.tftpd_output.Name = "tftpd_output";
            // 
            // FTPD
            // 
            this.FTPD.Controls.Add(this.ftp_dirtext1);
            this.FTPD.Controls.Add(this.ftp_dirbtn1);
            this.FTPD.Controls.Add(this.ftp_dirtext2);
            this.FTPD.Controls.Add(this.ftp_dirbtn2);
            this.FTPD.Controls.Add(this.ftp_dirtext3);
            this.FTPD.Controls.Add(this.ftp_dirbtn3);
            this.FTPD.Controls.Add(this.ftp_dirtext4);
            this.FTPD.Controls.Add(this.ftp_dirbtn4);
            this.FTPD.Controls.Add(this.ftp_dirtext5);
            this.FTPD.Controls.Add(this.ftp_dirbtn5);
            this.FTPD.Controls.Add(this.btn_ftp);
            resources.ApplyResources(this.FTPD, "FTPD");
            this.FTPD.Name = "FTPD";
            this.FTPD.UseVisualStyleBackColor = true;
            // 
            // ftp_dirtext1
            // 
            resources.ApplyResources(this.ftp_dirtext1, "ftp_dirtext1");
            this.ftp_dirtext1.Name = "ftp_dirtext1";
            // 
            // ftp_dirbtn1
            // 
            resources.ApplyResources(this.ftp_dirbtn1, "ftp_dirbtn1");
            this.ftp_dirbtn1.Name = "ftp_dirbtn1";
            this.ftp_dirbtn1.UseVisualStyleBackColor = true;
            // 
            // ftp_dirtext2
            // 
            resources.ApplyResources(this.ftp_dirtext2, "ftp_dirtext2");
            this.ftp_dirtext2.Name = "ftp_dirtext2";
            // 
            // ftp_dirbtn2
            // 
            resources.ApplyResources(this.ftp_dirbtn2, "ftp_dirbtn2");
            this.ftp_dirbtn2.Name = "ftp_dirbtn2";
            this.ftp_dirbtn2.UseVisualStyleBackColor = true;
            // 
            // ftp_dirtext3
            // 
            resources.ApplyResources(this.ftp_dirtext3, "ftp_dirtext3");
            this.ftp_dirtext3.Name = "ftp_dirtext3";
            // 
            // ftp_dirbtn3
            // 
            resources.ApplyResources(this.ftp_dirbtn3, "ftp_dirbtn3");
            this.ftp_dirbtn3.Name = "ftp_dirbtn3";
            this.ftp_dirbtn3.UseVisualStyleBackColor = true;
            // 
            // ftp_dirtext4
            // 
            resources.ApplyResources(this.ftp_dirtext4, "ftp_dirtext4");
            this.ftp_dirtext4.Name = "ftp_dirtext4";
            // 
            // ftp_dirbtn4
            // 
            resources.ApplyResources(this.ftp_dirbtn4, "ftp_dirbtn4");
            this.ftp_dirbtn4.Name = "ftp_dirbtn4";
            this.ftp_dirbtn4.UseVisualStyleBackColor = true;
            // 
            // ftp_dirtext5
            // 
            resources.ApplyResources(this.ftp_dirtext5, "ftp_dirtext5");
            this.ftp_dirtext5.Name = "ftp_dirtext5";
            // 
            // ftp_dirbtn5
            // 
            resources.ApplyResources(this.ftp_dirbtn5, "ftp_dirbtn5");
            this.ftp_dirbtn5.Name = "ftp_dirbtn5";
            this.ftp_dirbtn5.UseVisualStyleBackColor = true;
            // 
            // btn_ftp
            // 
            resources.ApplyResources(this.btn_ftp, "btn_ftp");
            this.btn_ftp.Name = "btn_ftp";
            this.btn_ftp.UseVisualStyleBackColor = true;
            // 
            // DHCPD
            // 
            this.DHCPD.Controls.Add(this.dhcp_startlabel);
            this.DHCPD.Controls.Add(this.dhcp_starttext);
            this.DHCPD.Controls.Add(this.dhcp_endlabel);
            this.DHCPD.Controls.Add(this.dhcp_endtext);
            this.DHCPD.Controls.Add(this.dhcp_gatewaylabel);
            this.DHCPD.Controls.Add(this.dhcp_gatewaytext);
            this.DHCPD.Controls.Add(this.dhcp_logbtn);
            this.DHCPD.Controls.Add(this.dhcp_logtext);
            this.DHCPD.Controls.Add(this.dhcp_btn);
            this.DHCPD.Controls.Add(this.dhcp_logmsg);
            resources.ApplyResources(this.DHCPD, "DHCPD");
            this.DHCPD.Name = "DHCPD";
            // 
            // dhcp_startlabel
            // 
            resources.ApplyResources(this.dhcp_startlabel, "dhcp_startlabel");
            this.dhcp_startlabel.BackColor = System.Drawing.Color.Transparent;
            this.dhcp_startlabel.Name = "dhcp_startlabel";
            // 
            // dhcp_starttext
            // 
            resources.ApplyResources(this.dhcp_starttext, "dhcp_starttext");
            this.dhcp_starttext.Name = "dhcp_starttext";
            // 
            // dhcp_endlabel
            // 
            resources.ApplyResources(this.dhcp_endlabel, "dhcp_endlabel");
            this.dhcp_endlabel.BackColor = System.Drawing.Color.Transparent;
            this.dhcp_endlabel.Name = "dhcp_endlabel";
            // 
            // dhcp_endtext
            // 
            resources.ApplyResources(this.dhcp_endtext, "dhcp_endtext");
            this.dhcp_endtext.Name = "dhcp_endtext";
            // 
            // dhcp_gatewaylabel
            // 
            resources.ApplyResources(this.dhcp_gatewaylabel, "dhcp_gatewaylabel");
            this.dhcp_gatewaylabel.BackColor = System.Drawing.Color.Transparent;
            this.dhcp_gatewaylabel.Name = "dhcp_gatewaylabel";
            // 
            // dhcp_gatewaytext
            // 
            resources.ApplyResources(this.dhcp_gatewaytext, "dhcp_gatewaytext");
            this.dhcp_gatewaytext.Name = "dhcp_gatewaytext";
            // 
            // dhcp_logbtn
            // 
            this.dhcp_logbtn.BackColor = System.Drawing.Color.YellowGreen;
            resources.ApplyResources(this.dhcp_logbtn, "dhcp_logbtn");
            this.dhcp_logbtn.FlatAppearance.BorderColor = System.Drawing.Color.FromArgb(((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))));
            this.dhcp_logbtn.FlatAppearance.BorderSize = 0;
            this.dhcp_logbtn.FlatAppearance.MouseDownBackColor = System.Drawing.Color.Transparent;
            this.dhcp_logbtn.FlatAppearance.MouseOverBackColor = System.Drawing.Color.Transparent;
            this.dhcp_logbtn.ForeColor = System.Drawing.SystemColors.ControlText;
            this.dhcp_logbtn.Name = "dhcp_logbtn";
            this.dhcp_logbtn.UseVisualStyleBackColor = false;
            // 
            // dhcp_logtext
            // 
            resources.ApplyResources(this.dhcp_logtext, "dhcp_logtext");
            this.dhcp_logtext.Name = "dhcp_logtext";
            // 
            // dhcp_btn
            // 
            resources.ApplyResources(this.dhcp_btn, "dhcp_btn");
            this.dhcp_btn.BackColor = System.Drawing.Color.YellowGreen;
            this.dhcp_btn.FlatAppearance.BorderColor = System.Drawing.Color.FromArgb(((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))));
            this.dhcp_btn.FlatAppearance.BorderSize = 0;
            this.dhcp_btn.FlatAppearance.MouseDownBackColor = System.Drawing.Color.Transparent;
            this.dhcp_btn.FlatAppearance.MouseOverBackColor = System.Drawing.Color.Transparent;
            this.dhcp_btn.ForeColor = System.Drawing.SystemColors.ControlText;
            this.dhcp_btn.Name = "dhcp_btn";
            this.dhcp_btn.UseVisualStyleBackColor = false;
            // 
            // dhcp_logmsg
            // 
            resources.ApplyResources(this.dhcp_logmsg, "dhcp_logmsg");
            this.dhcp_logmsg.FormattingEnabled = true;
            this.dhcp_logmsg.Name = "dhcp_logmsg";
            // 
            // Setting
            // 
            this.Setting.Controls.Add(this.lang);
            this.Setting.Controls.Add(this.lang_opt);
            this.Setting.Controls.Add(this.setting_output);
            resources.ApplyResources(this.Setting, "Setting");
            this.Setting.Name = "Setting";
            this.Setting.UseVisualStyleBackColor = true;
            // 
            // lang
            // 
            resources.ApplyResources(this.lang, "lang");
            this.lang.Name = "lang";
            // 
            // lang_opt
            // 
            this.lang_opt.FormattingEnabled = true;
            resources.ApplyResources(this.lang_opt, "lang_opt");
            this.lang_opt.Name = "lang_opt";
            // 
            // setting_output
            // 
            this.setting_output.FormattingEnabled = true;
            resources.ApplyResources(this.setting_output, "setting_output");
            this.setting_output.Name = "setting_output";
            // 
            // Form1
            // 
            resources.ApplyResources(this, "$this");
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.Controls.Add(this.toolStripContainer1);
            this.DoubleBuffered = true;
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
            this.FTPD.ResumeLayout(false);
            this.FTPD.PerformLayout();
            this.DHCPD.ResumeLayout(false);
            this.DHCPD.PerformLayout();
            this.Setting.ResumeLayout(false);
            this.Setting.PerformLayout();
            this.ResumeLayout(false);

        }

        #endregion

        private System.Windows.Forms.Label label_interval;
        private System.Windows.Forms.Label label_count;
        private System.Windows.Forms.TextBox text_addr;
        private System.Windows.Forms.TextBox text_interval;
        private System.Windows.Forms.ToolStripContainer toolStripContainer1;
        private System.Windows.Forms.Button btn_ping;
        private System.Windows.Forms.Button btn_ping_log;
        private System.Windows.Forms.TextBox text_count;
        private System.Windows.Forms.Label label_addr;
        private System.Windows.Forms.TextBox text_logpath;
        private System.Windows.Forms.TabControl tabs;
        private System.Windows.Forms.TabPage Ping;
        private System.Windows.Forms.TabPage HTTPD;
        private System.Windows.Forms.ListBox ping_output;
        private System.Windows.Forms.TabPage FTPD;
        private System.Windows.Forms.TabPage Setting;
        private System.Windows.Forms.Label label_http_port;
        private System.Windows.Forms.TextBox text_http_port;
        private System.Windows.Forms.TextBox text_http_dir;
        private System.Windows.Forms.Button btn_httpd;
        private System.Windows.Forms.TextBox ftp_dirtext1;
        private System.Windows.Forms.Button ftp_dirbtn1;
        private System.Windows.Forms.TextBox ftp_dirtext2;
        private System.Windows.Forms.Button ftp_dirbtn2;
        private System.Windows.Forms.TextBox ftp_dirtext3;
        private System.Windows.Forms.Button ftp_dirbtn3;
        private System.Windows.Forms.TextBox ftp_dirtext4;
        private System.Windows.Forms.Button ftp_dirbtn4;
        private System.Windows.Forms.TextBox ftp_dirtext5;
        private System.Windows.Forms.Button ftp_dirbtn5;
        private System.Windows.Forms.Button btn_ftp;
        private System.Windows.Forms.TabPage TFTPD;
        private System.Windows.Forms.TextBox tftp_dirtext1;
        private System.Windows.Forms.Button tftp_dirbtn1;
        private System.Windows.Forms.TextBox tftp_dirtext2;
        private System.Windows.Forms.Button tftp_dirbtn2;
        private System.Windows.Forms.TextBox tftp_dirtext3;
        private System.Windows.Forms.Button tftp_dirbtn3;
        private System.Windows.Forms.TextBox tftp_dirtext4;
        private System.Windows.Forms.Button tftp_dirbtn4;
        private System.Windows.Forms.TextBox tftp_dirtext5;
        private System.Windows.Forms.Button tftp_dirbtn5;
        private System.Windows.Forms.Label lang;
        private System.Windows.Forms.ComboBox lang_opt;
        private System.Windows.Forms.ListBox httpd_output;
        private System.Windows.Forms.TabPage DHCPD;
        private System.Windows.Forms.Label dhcp_startlabel;
        private System.Windows.Forms.TextBox dhcp_starttext;
        private System.Windows.Forms.Label dhcp_endlabel;
        private System.Windows.Forms.TextBox dhcp_endtext;
        private System.Windows.Forms.Label dhcp_gatewaylabel;
        private System.Windows.Forms.TextBox dhcp_gatewaytext;
        private System.Windows.Forms.Button dhcp_logbtn;
        private System.Windows.Forms.TextBox dhcp_logtext;
        private System.Windows.Forms.Button dhcp_btn;
        private System.Windows.Forms.ListBox dhcp_logmsg;
        private System.Windows.Forms.Button btn_http_dir;
        private System.Windows.Forms.ListBox tftpd_output;
        private System.Windows.Forms.Button tftpd_btn;
        private System.Windows.Forms.TabPage Scan;
        private System.Windows.Forms.Label label_scanip;
        private System.Windows.Forms.TextBox text_scanstart;
        private System.Windows.Forms.Label label_scanend;
        private System.Windows.Forms.TextBox text_scanend;
        private System.Windows.Forms.Button btn_scan;
        private System.Windows.Forms.FlowLayoutPanel fp_scan;
        private System.Windows.Forms.ListBox setting_output;
    }
}

