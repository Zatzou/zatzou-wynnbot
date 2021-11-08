use serenity::model::channel::Message;

pub fn parse_command_args(msg: &Message) -> Vec<&str> {
    msg.content.split_whitespace().into_iter().collect()
}

pub fn parse_command_args_raw(msg: &Message) -> Option<&str> {
    Some(msg.content.split_once(' ')?.1)
}
