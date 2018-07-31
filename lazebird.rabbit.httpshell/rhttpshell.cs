using SharpShell.Attributes;
using SharpShell.SharpContextMenu;
using System;
using System.IO;
using System.IO.Pipes;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace lazebird.rabbit.httpshell
{
    [ComVisible(true)]
    [COMServerAssociation(AssociationType.AllFiles), COMServerAssociation(AssociationType.Directory)]
    public class rhttpshell : SharpContextMenu
    {
        protected override bool CanShowMenu()
        {
            return true;
        }

        protected override ContextMenuStrip CreateMenu()
        {
            ContextMenuStrip menu = new ContextMenuStrip();
            ToolStripMenuItem item = new ToolStripMenuItem
            {
                Text = "Add to Rabbit.http",
                Image = Properties.Resources.CountLines
            }; 
            item.Click += Item_Click;
            //设置图像及位置
            //item.Image = XXX
            //item.ImageAlign = System.Drawing.ContentAlignment.MiddleLeft
            menu.Items.Add(item);
            return menu;
        }
        void Item_Click(object sender, EventArgs e)
        {
            try
            {
                NamedPipeClientStream pipeClient = new NamedPipeClientStream("localhost", "lazebird.rabbit.rabbit", PipeDirection.Out);
                pipeClient.Connect(3000);
                StreamWriter sw = new StreamWriter(pipeClient);
                foreach (var path in SelectedItemPaths) sw.WriteLine(path);
            }
            catch (Exception) { }
        }
    }
}
