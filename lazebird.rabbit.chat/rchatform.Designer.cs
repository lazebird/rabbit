namespace lazebird.rabbit.chat
{
    partial class rchatform
    {
        /// <summary>
        /// Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
            chatss.Dispose();
        }

        #region Windows Form Designer generated code

        /// <summary>
        /// Required method for Designer support - do not modify
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            this.pa_chat = new System.Windows.Forms.Panel();
            this.rtb_chat = new System.Windows.Forms.RichTextBox();
            this.SuspendLayout();
            // 
            // pa_chat
            // 
            this.pa_chat.Anchor = ((System.Windows.Forms.AnchorStyles)((((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Bottom) 
            | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.pa_chat.AutoScroll = true;
            this.pa_chat.BackColor = System.Drawing.Color.Gray;
            this.pa_chat.Font = new System.Drawing.Font("Microsoft YaHei UI", 12F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.pa_chat.Location = new System.Drawing.Point(3, 2);
            this.pa_chat.Name = "pa_chat";
            this.pa_chat.Size = new System.Drawing.Size(781, 396);
            this.pa_chat.TabIndex = 0;
            // 
            // rtb_chat
            // 
            this.rtb_chat.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Bottom | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.rtb_chat.Font = new System.Drawing.Font("Microsoft YaHei UI", 12F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.rtb_chat.Location = new System.Drawing.Point(3, 404);
            this.rtb_chat.Name = "rtb_chat";
            this.rtb_chat.Size = new System.Drawing.Size(781, 157);
            this.rtb_chat.TabIndex = 1;
            this.rtb_chat.Text = "";
            // 
            // rchatform
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(784, 561);
            this.Controls.Add(this.pa_chat);
            this.Controls.Add(this.rtb_chat);
            this.Name = "rchatform";
            this.Text = "chatform";
            this.ResumeLayout(false);

        }

        #endregion

        private System.Windows.Forms.Panel pa_chat;
        private System.Windows.Forms.RichTextBox rtb_chat;
    }
}