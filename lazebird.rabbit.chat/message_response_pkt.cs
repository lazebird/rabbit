namespace lazebird.rabbit.chat
{
    class message_response_pkt : pkt
    {
        public message_response_pkt(string user, string id)
        {
            this.type = "message_response";
            this.user = user;
            this.id = id;
        }
    }
}
