namespace lazebird.rabbit.chat
{
    class query_response_pkt : pkt
    {
        public query_response_pkt(string user)
        {
            this.type = "query_response";
            this.user = user;
        }
    }
}
