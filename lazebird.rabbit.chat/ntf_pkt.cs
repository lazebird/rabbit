namespace lazebird.rabbit.chat
{
    class ntf_pkt : pkt
    {
        public ntf_pkt(string user, string msg)
        {
            this.type = "notification";
            this.user = user;
            content = msg;
        }
    }
}
