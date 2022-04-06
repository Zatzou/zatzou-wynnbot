# Zatzou's wynnbot
My very amazing discord bot that does stuff with wynncraft

## Adding the bot to a server
You can either invite the bot I run from [here](https://discord.com/api/oauth2/authorize?client_id=896856896382844938&permissions=0&scope=bot%20applications.commands) or build the bot yourself and self host it

## Building and running
1. Install [rust](https://rustup.rs/)
2. clone the repo
3. run `cargo build --release`
4. copy the executable from ./target/release/ and the resources folder from the project root to somewhere
5. make a config file called config.toml (the example_config.toml contains all of the variables)
6. run the bot
7. invite the bot to your server and grant it the oauth scopes of `bot` and `application.commands` you can easily do this in the oauth2 URL generator (the bot will work with only the bot scope but slash commands will only work with the `application.commands` scope enabled)


## Commands:
- /help
- /map
- /gather [material]
- /up (server number)
- /sp
- /id [wynntils id string]
- /maxid [item name]
