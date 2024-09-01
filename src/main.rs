use std::env;

use serenity::async_trait;
use serenity::model::channel::{Message, MessageReference};
use serenity::model::id::{ChannelId, MessageId};
use serenity::model::gateway::Ready;
use serenity::builder::{CreateAttachment, CreateMessage};
use serenity::Result as SerenityResult;
use serenity::utils::MessageBuilder;
use serenity::prelude::*;
use serenity::utils::FormattedTimestamp;

struct Handler;

impl Handler{

    async fn handle_spoilme(ctx: Context, channel_id: ChannelId, message_id: MessageId) ->
        SerenityResult<()>{

        // TODO: this errors out if there are reactions!! error about burst_colours
        let ref_msg = channel_id.message(&ctx, message_id).await?;
        //println!("Got referenced message: {:?}\n\n",ref_msg);

        let mut attachments: Vec<CreateAttachment> = vec!();
        for attachment in &ref_msg.attachments{
            let mut spoiled_attachment = CreateAttachment::url(&ctx, &attachment.url).await?;
            if ! spoiled_attachment.filename.starts_with("SPOILER_") {
                spoiled_attachment.filename = format!("SPOILER_{}",spoiled_attachment.filename);
            }
            attachments.push(spoiled_attachment);

        }

        let mut builder = MessageBuilder::new();
        builder.mention(&ref_msg.author)
            .push(format!("\n_ {}",FormattedTimestamp::from(ref_msg.timestamp)));
        if let Some(edit_ts) = ref_msg.edited_timestamp{
            builder.push(" (edited at ")
            .push(FormattedTimestamp::from(edit_ts).to_string())
            .push(")");
        }
        builder.push('_'); // end italic
        if !ref_msg.content.is_empty() {
            builder.push('\n').push_quote(&ref_msg.content);
        }

        channel_id.send_files(&ctx, attachments,
                              CreateMessage::new().content(builder.build()))
            .await?;
        ref_msg.delete(&ctx).await
    }
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.

    async fn message(&self, ctx: Context, msg: Message) {
        //println!("{:?}\n\n", msg);
        if msg.content == "!spoilme" {
            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.

            match msg.message_reference{
                Some(MessageReference{channel_id, message_id, ..}) =>{
                    if let Some(message_id) = message_id{
                        if let Err(why) = Self::handle_spoilme(ctx, channel_id, message_id).await{
                            println!("Failed to handle spoilme: {}", why);
                        }
                    }
                }
                _ => ()
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
