﻿namespace lazebird.rabbit.chat
{
    partial class rntfform
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
        }

        #region Windows Form Designer generated code

        /// <summary>
        /// Required method for Designer support - do not modify
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            this.rtb_chat = new System.Windows.Forms.RichTextBox();
            this.pa_chat = new System.Windows.Forms.Panel();
            this.SuspendLayout();
            // 
            // rtb_chat
            // 
            this.rtb_chat.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Bottom | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.rtb_chat.Location = new System.Drawing.Point(10, 400);
            this.rtb_chat.Name = "rtb_chat";
            this.rtb_chat.Size = new System.Drawing.Size(770, 160);
            this.rtb_chat.TabIndex = 0;
            this.rtb_chat.Text = "";
            // 
            // pa_chat
            // 
            this.pa_chat.Anchor = ((System.Windows.Forms.AnchorStyles)((((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Bottom) 
            | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.pa_chat.BackColor = System.Drawing.Color.Gray;
            this.pa_chat.Location = new System.Drawing.Point(10, 3);
            this.pa_chat.Name = "pa_chat";
            this.pa_chat.Size = new System.Drawing.Size(770, 395);
            this.pa_chat.TabIndex = 1;
            // 
            // rntfform
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(784, 561);
            this.Controls.Add(this.pa_chat);
            this.Controls.Add(this.rtb_chat);
            this.Name = "rntfform";
            this.Text = "rntfform";
            this.ResumeLayout(false);

        }

        #endregion

        private System.Windows.Forms.RichTextBox rtb_chat;
        private System.Windows.Forms.Panel pa_chat;
    }
}