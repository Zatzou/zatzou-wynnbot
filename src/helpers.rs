use serenity::model::channel::Message;

pub fn parse_command_args(msg: &Message) -> Vec<&str> {
    msg.content.split_whitespace().into_iter().collect()
}