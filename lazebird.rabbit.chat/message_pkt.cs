namespace lazebird.rabbit.chat
{
    class message_pkt:pkt
    {
        public message_pkt(string user, string id, string content)
        {
            this.type = "message";
            this.user = user;
            this.id = id;
            this.content = content;
        }
    }
}
