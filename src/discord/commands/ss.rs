use crate::types::discord::framework::Error;
use serenity::all::CreateThread;

use super::Context;

pub async fn handle(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    ctx.say("スレッドを作成します").await?;
    let builder = CreateThread::new(format!("SSくん-{}", ctx.id()))
        .kind(serenity::all::ChannelType::PublicThread);
    let thread = ctx.channel_id().create_thread(ctx.http(), builder).await?;
    thread.id.say(ctx.http(), "質問してみましょう").await?;

    Ok(())
}
