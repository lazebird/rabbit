namespace ping
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
            this.label_interval = new System.Windows.Forms.Label();
            this.label_count = new System.Windows.Forms.Label();
            this.text_addr = new System.Windows.Forms.TextBox();
            this.text_interval = new System.Windows.Forms.TextBox();
            this.toolStripContainer1 = new System.Windows.Forms.ToolStripContainer();
            this.tabs = new System.Windows.Forms.TabControl();
            this.Ping = new System.Windows.Forms.TabPage();
            this.output = new System.Windows.Forms.ListBox();
            this.btn_ping = new System.Windows.Forms.Button();
            this.btn_log = new System.Windows.Forms.Button();
            this.text_logpath = new System.Windows.Forms.TextBox();
            this.label_addr = new System.Windows.Forms.Label();
            this.text_count = new System.Windows.Forms.TextBox();
            this.HTTPD = new System.Windows.Forms.TabPage();
            this.Setting = new System.Windows.Forms.TabPage();
            this.FTPD = new System.Windows.Forms.TabPage();
            this.TFTPD = new System.Windows.Forms.TabPage();
            this.toolStripContainer1.ContentPanel.SuspendLayout();
            this.toolStripContainer1.SuspendLayout();
            this.tabs.SuspendLayout();
            this.Ping.SuspendLayout();
            this.SuspendLayout();
            // 
            // label_interval
            // 
            this.label_interval.AutoSize = true;
            this.label_interval.BackColor = System.Drawing.Color.Transparent;
            this.label_interval.Location = new System.Drawing.Point(5, 60);
            this.label_interval.Margin = new System.Windows.Forms.Padding(2, 0, 2, 0);
            this.label_interval.Name = "label_interval";
            this.label_interval.Size = new System.Drawing.Size(66, 20);
            this.label_interval.TabIndex = 0;
            this.label_interval.Text = "间隔(ms)";
            // 
            // label_count
            // 
            this.label_count.AutoSize = true;
            this.label_count.BackColor = System.Drawing.Color.Transparent;
            this.label_count.Location = new System.Drawing.Point(5, 101);
            this.label_count.Margin = new System.Windows.Forms.Padding(2, 0, 2, 0);
            this.label_count.Name = "label_count";
            this.label_count.Size = new System.Drawing.Size(37, 20);
            this.label_count.TabIndex = 1;
            this.label_count.Text = "次数";
            // 
            // text_addr
            // 
            this.text_addr.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.text_addr.Location = new System.Drawing.Point(72, 14);
            this.text_addr.Margin = new System.Windows.Forms.Padding(2);
            this.text_addr.Name = "text_addr";
            this.text_addr.Size = new System.Drawing.Size(677, 25);
            this.text_addr.TabIndex = 1;
            this.text_addr.Text = "www.mozilla.com";
            // 
            // text_interval
            // 
            this.text_interval.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.text_interval.Location = new System.Drawing.Point(72, 55);
            this.text_interval.Margin = new System.Windows.Forms.Padding(2);
            this.text_interval.Name = "text_interval";
            this.text_interval.Size = new System.Drawing.Size(677, 25);
            this.text_interval.TabIndex = 2;
            this.text_interval.Text = "1000";
            // 
            // toolStripContainer1
            // 
            this.toolStripContainer1.BottomToolStripPanelVisible = false;
            // 
            // toolStripContainer1.ContentPanel
            // 
            this.toolStripContainer1.ContentPanel.AutoScroll = true;
            this.toolStripContainer1.ContentPanel.BackgroundImage = ((System.Drawing.Image)(resources.GetObject("toolStripContainer1.ContentPanel.BackgroundImage")));
            this.toolStripContainer1.ContentPanel.BackgroundImageLayout = System.Windows.Forms.ImageLayout.Stretch;
            this.toolStripContainer1.ContentPanel.Controls.Add(this.tabs);
            this.toolStripContainer1.ContentPanel.Margin = new System.Windows.Forms.Padding(2);
            this.toolStripContainer1.ContentPanel.Size = new System.Drawing.Size(767, 477);
            this.toolStripContainer1.Dock = System.Windows.Forms.DockStyle.Fill;
            this.toolStripContainer1.LeftToolStripPanelVisible = false;
            this.toolStripContainer1.Location = new System.Drawing.Point(0, 0);
            this.toolStripContainer1.Margin = new System.Windows.Forms.Padding(2);
            this.toolStripContainer1.Name = "toolStripContainer1";
            this.toolStripContainer1.RightToolStripPanelVisible = false;
            this.toolStripContainer1.Size = new System.Drawing.Size(767, 477);
            this.toolStripContainer1.TabIndex = 8;
            this.toolStripContainer1.Text = "toolStripContainer1";
            this.toolStripContainer1.TopToolStripPanelVisible = false;
            // 
            // tabs
            // 
            this.tabs.Controls.Add(this.Ping);
            this.tabs.Controls.Add(this.HTTPD);
            this.tabs.Controls.Add(this.TFTPD);
            this.tabs.Controls.Add(this.FTPD);
            this.tabs.Controls.Add(this.Setting);
            this.tabs.Location = new System.Drawing.Point(0, 0);
            this.tabs.Name = "tabs";
            this.tabs.SelectedIndex = 0;
            this.tabs.Size = new System.Drawing.Size(765, 475);
            this.tabs.TabIndex = 13;
            // 
            // Ping
            // 
            this.Ping.Controls.Add(this.label_addr);
            this.Ping.Controls.Add(this.text_addr);
            this.Ping.Controls.Add(this.label_interval);
            this.Ping.Controls.Add(this.text_interval);
            this.Ping.Controls.Add(this.label_count);
            this.Ping.Controls.Add(this.text_count);
            this.Ping.Controls.Add(this.btn_log);
            this.Ping.Controls.Add(this.text_logpath);
            this.Ping.Controls.Add(this.btn_ping);
            this.Ping.Controls.Add(this.output);
            this.Ping.Location = new System.Drawing.Point(4, 29);
            this.Ping.Name = "Ping";
            this.Ping.Padding = new System.Windows.Forms.Padding(3);
            this.Ping.Size = new System.Drawing.Size(757, 442);
            this.Ping.TabIndex = 0;
            this.Ping.Text = "Ping";
            this.Ping.UseVisualStyleBackColor = true;
            // 
            // output
            // 
            this.output.Anchor = ((System.Windows.Forms.AnchorStyles)((((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Bottom) 
            | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.output.Font = new System.Drawing.Font("Microsoft YaHei UI", 7.8F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.output.FormattingEnabled = true;
            this.output.ItemHeight = 19;
            this.output.Location = new System.Drawing.Point(9, 253);
            this.output.Margin = new System.Windows.Forms.Padding(2);
            this.output.Name = "output";
            this.output.Size = new System.Drawing.Size(740, 175);
            this.output.TabIndex = 7;
            // 
            // btn_ping
            // 
            this.btn_ping.Anchor = ((System.Windows.Forms.AnchorStyles)((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Right)));
            this.btn_ping.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_ping.FlatAppearance.BorderColor = System.Drawing.Color.FromArgb(((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))));
            this.btn_ping.FlatAppearance.BorderSize = 0;
            this.btn_ping.FlatAppearance.MouseDownBackColor = System.Drawing.Color.Transparent;
            this.btn_ping.FlatAppearance.MouseOverBackColor = System.Drawing.Color.Transparent;
            this.btn_ping.FlatStyle = System.Windows.Forms.FlatStyle.Flat;
            this.btn_ping.Font = new System.Drawing.Font("Microsoft YaHei UI", 7.8F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.btn_ping.ForeColor = System.Drawing.SystemColors.ControlText;
            this.btn_ping.Location = new System.Drawing.Point(676, 190);
            this.btn_ping.Margin = new System.Windows.Forms.Padding(2);
            this.btn_ping.Name = "btn_ping";
            this.btn_ping.Size = new System.Drawing.Size(73, 37);
            this.btn_ping.TabIndex = 6;
            this.btn_ping.Text = "开始";
            this.btn_ping.UseVisualStyleBackColor = false;
            this.btn_ping.Click += new System.EventHandler(this.button1_Click);
            // 
            // btn_log
            // 
            this.btn_log.BackColor = System.Drawing.Color.YellowGreen;
            this.btn_log.BackgroundImageLayout = System.Windows.Forms.ImageLayout.None;
            this.btn_log.FlatAppearance.BorderColor = System.Drawing.Color.FromArgb(((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))), ((int)(((byte)(0)))));
            this.btn_log.FlatAppearance.BorderSize = 0;
            this.btn_log.FlatAppearance.MouseDownBackColor = System.Drawing.Color.Transparent;
            this.btn_log.FlatAppearance.MouseOverBackColor = System.Drawing.Color.Transparent;
            this.btn_log.FlatStyle = System.Windows.Forms.FlatStyle.Flat;
            this.btn_log.Font = new System.Drawing.Font("Microsoft YaHei UI", 7.8F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.btn_log.ForeColor = System.Drawing.SystemColors.ControlText;
            this.btn_log.ImageAlign = System.Drawing.ContentAlignment.TopLeft;
            this.btn_log.Location = new System.Drawing.Point(9, 133);
            this.btn_log.Margin = new System.Windows.Forms.Padding(2);
            this.btn_log.Name = "btn_log";
            this.btn_log.Size = new System.Drawing.Size(52, 32);
            this.btn_log.TabIndex = 4;
            this.btn_log.Text = "日志";
            this.btn_log.UseVisualStyleBackColor = false;
            this.btn_log.Click += new System.EventHandler(this.button2_Click);
            // 
            // text_logpath
            // 
            this.text_logpath.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.text_logpath.Location = new System.Drawing.Point(72, 140);
            this.text_logpath.Margin = new System.Windows.Forms.Padding(2);
            this.text_logpath.Name = "text_logpath";
            this.text_logpath.Size = new System.Drawing.Size(677, 25);
            this.text_logpath.TabIndex = 5;
            // 
            // label_addr
            // 
            this.label_addr.AutoSize = true;
            this.label_addr.BackColor = System.Drawing.Color.Transparent;
            this.label_addr.Location = new System.Drawing.Point(5, 19);
            this.label_addr.Margin = new System.Windows.Forms.Padding(2, 0, 2, 0);
            this.label_addr.Name = "label_addr";
            this.label_addr.Size = new System.Drawing.Size(37, 20);
            this.label_addr.TabIndex = 12;
            this.label_addr.Text = "地址";
            // 
            // text_count
            // 
            this.text_count.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.text_count.Location = new System.Drawing.Point(72, 96);
            this.text_count.Margin = new System.Windows.Forms.Padding(2);
            this.text_count.Name = "text_count";
            this.text_count.Size = new System.Drawing.Size(677, 25);
            this.text_count.TabIndex = 3;
            this.text_count.Text = "-1";
            // 
            // HTTPD
            // 
            this.HTTPD.Location = new System.Drawing.Point(4, 29);
            this.HTTPD.Name = "HTTPD";
            this.HTTPD.Padding = new System.Windows.Forms.Padding(3);
            this.HTTPD.Size = new System.Drawing.Size(757, 442);
            this.HTTPD.TabIndex = 1;
            this.HTTPD.Text = "HTTPD";
            this.HTTPD.UseVisualStyleBackColor = true;
            // 
            // Setting
            // 
            this.Setting.Location = new System.Drawing.Point(4, 29);
            this.Setting.Name = "Setting";
            this.Setting.Padding = new System.Windows.Forms.Padding(3);
            this.Setting.Size = new System.Drawing.Size(757, 442);
            this.Setting.TabIndex = 2;
            this.Setting.Text = "Setting";
            this.Setting.UseVisualStyleBackColor = true;
            // 
            // FTPD
            // 
            this.FTPD.Location = new System.Drawing.Point(4, 29);
            this.FTPD.Name = "FTPD";
            this.FTPD.Padding = new System.Windows.Forms.Padding(3);
            this.FTPD.Size = new System.Drawing.Size(757, 442);
            this.FTPD.TabIndex = 3;
            this.FTPD.Text = "FTPD";
            this.FTPD.UseVisualStyleBackColor = true;
            // 
            // TFTPD
            // 
            this.TFTPD.Location = new System.Drawing.Point(4, 29);
            this.TFTPD.Name = "TFTPD";
            this.TFTPD.Padding = new System.Windows.Forms.Padding(3);
            this.TFTPD.Size = new System.Drawing.Size(757, 442);
            this.TFTPD.TabIndex = 4;
            this.TFTPD.Text = "TFTPD";
            this.TFTPD.UseVisualStyleBackColor = true;
            // 
            // Form1
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(8F, 20F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.BackgroundImage = ((System.Drawing.Image)(resources.GetObject("$this.BackgroundImage")));
            this.BackgroundImageLayout = System.Windows.Forms.ImageLayout.Zoom;
            this.ClientSize = new System.Drawing.Size(767, 477);
            this.Controls.Add(this.toolStripContainer1);
            this.DoubleBuffered = true;
            this.Font = new System.Drawing.Font("Microsoft YaHei UI", 8.25F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point, ((byte)(134)));
            this.Icon = ((System.Drawing.Icon)(resources.GetObject("$this.Icon")));
            this.Margin = new System.Windows.Forms.Padding(2);
            this.Name = "Form1";
            this.Text = "Rabbit";
            this.toolStripContainer1.ContentPanel.ResumeLayout(false);
            this.toolStripContainer1.ResumeLayout(false);
            this.toolStripContainer1.PerformLayout();
            this.tabs.ResumeLayout(false);
            this.Ping.ResumeLayout(false);
            this.Ping.PerformLayout();
            this.ResumeLayout(false);

        }

        #endregion

        private System.Windows.Forms.Label label_interval;
        private System.Windows.Forms.Label label_count;
        private System.Windows.Forms.TextBox text_addr;
        private System.Windows.Forms.TextBox text_interval;
        private System.Windows.Forms.ToolStripContainer toolStripContainer1;
        private System.Windows.Forms.Button btn_ping;
        private System.Windows.Forms.Button btn_log;
        private System.Windows.Forms.TextBox text_count;
        private System.Windows.Forms.Label label_addr;
        private System.Windows.Forms.TextBox text_logpath;
        private System.Windows.Forms.TabControl tabs;
        private System.Windows.Forms.TabPage Ping;
        private System.Windows.Forms.TabPage HTTPD;
        private System.Windows.Forms.ListBox output;
        private System.Windows.Forms.TabPage TFTPD;
        private System.Windows.Forms.TabPage FTPD;
        private System.Windows.Forms.TabPage Setting;
    }
}

