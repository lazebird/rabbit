# rkey

## 描述
- 快捷键管理库

## 目标
- 支持多标签页多快捷键注册和管理

## API
1. public rkey()
    - 构造函数
 
2. public bool exec(int tabindex, Keys k)
    - 快捷键执行函数；按照输入参数，找出匹配的快捷键回调接口，执行回调
    - tabindex：当前tab索引
    - k：当前的按键

3. public bool bind(int tabindex, Keys k, Action<Keys> f)
    - 快捷键注册/绑定接口；提供快捷键的tab页，按键，和回调函数
    - tabindex：快捷键所处tab页
    - k：快捷键的键值
    - f：快捷键触发时的回调函数
    - 本接口派生了多个其他回调函数类型的接口，功能相似，不一一列出

## 示例
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