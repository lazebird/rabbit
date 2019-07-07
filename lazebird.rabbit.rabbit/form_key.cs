using lazebird.rabbit.key;
using System.Windows.Forms;

namespace lazebird.rabbit.rabbit
{
    public partial class Form1 : Form
    {
        rkey key;
        void init_form_key()
        {
            key = new rkey();
            key.bind(key.globalindex, Keys.Escape, Close);
        }
        protected override bool ProcessDialogKey(Keys keyData)
        {
            if (key.exec(tabs.SelectedIndex, keyData)) return true;
            return base.ProcessDialogKey(keyData);
        }
    }
}
