# rchat

## 描述

## 目标

## API
1. public rchat(Action<string> log, Action<IPEndPoint, string> adduser)
    - 构造函数
    - log：log输出接口
    - adduser: 发现新用户时的接口；后续考虑改为事件委托方式

2. public void set_name(string name)
    - 设置用户昵称
    - name: 用户昵称

3. public void new_query(string ip, int port)
    - 发起新的用户扫描
    - ip: 广播IP，通常为255.255.255.255
    - port：广播端口

4. public void new_chat(IPEndPoint r, string ruser)
    - 发起新的聊天
    - r: 对端连接信息
    - ruser: 对端用户名

5. public void new_notification(string ip, int port)
    - 发起广播消息
    - ip: 广播IP，通常为255.255.255.255
    - port：广播端口
6. public void start(int port)
    - 启动聊天服务侦听
    - port: 聊天侦听端口

7. public void stop()
    - 关闭聊天服务

## 示例
    ```
    chat = new rchat(chat_log_func, add_user);
    chat.start(1314);
   ```