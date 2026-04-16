# rkey

## Description
- Shortcut Management Library

## Target
- Support multiple tabs with multiple shortcuts for registration and management

## API
1. Public rkey()
    - Constructor
 
2. public bool exec(int tabindex, Keys k)
    - Shortcut key execution function; according to the input parameters, find the matching shortcut key callback interface, perform callback
    - tabindex: current tab index
    - k: current button

3. public bool bind(int tabindex, Keys k, Action<Keys> f)
    - Shortcut key registration/binding interface; tab page, keystroke, and callback function for providing shortcut keys
    - tabindex: tab page where the shortcut is located
    -k: key value of the shortcut key
    - f: callback function when the shortcut key is triggered
    - This interface derives several interfaces of other callback function types, which are similar in function and are not listed one by one.

## Sample
```
    rkey key;
    key = new rkey();
    key.bind(key.globalindex, Keys.Escape, Close);
    protected override bool ProcessDialogKey(Keys keyData)
    {
        if (key.exec(tabs.SelectedIndex, keyData)) return true;
        return base.ProcessDialogKey(keyData);
    }
```