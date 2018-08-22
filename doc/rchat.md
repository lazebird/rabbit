# rchat

## Description

## Target

## API
1. public rchat(Action<string> log, Action<IPEndPoint, string> adduser)
    - Constructor
    - log：log print function
    - adduser: user add function, called when an new user is found in lan. Consider changing to the event delegation method

2. public void set_name(string name)
    - Set user nickname
    - name: user nickname

3. public void new_query(string ip, int port)
    - Start a new user scan
    - ip: Broadcast IP, usually 255.255.255.255
    - port：Broadcast port

4. public void new_chat(IPEndPoint r, string ruser)
    - Start a new chat
    - r: Peer connection information
    - ruser: Peer username

5. public void new_notification(string ip, int port)
    - Start a broadcast message
    - ip: Broadcast IP, usually 255.255.255.255
    - port：Broadcast port

6. public void start(int port)
    - Start chat service 
    - port: Chat listening port

7. public void stop()
    - Stop chat service

## 示例
    ```
    chat = new rchat(chat_log_func, add_user);
    chat.start(1314);
   ```