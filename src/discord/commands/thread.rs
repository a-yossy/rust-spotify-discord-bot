use crate::{
    model::thread::{self, Thread},
    types::discord::framework::Error,
};
use anyhow::Context as AnyhowContext;
use serenity::all::CreateThread;

use super::Context;

pub async fn handle(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    ctx.say("スレッドを作成します").await?;
    let builder = CreateThread::new(format!("SSくん-{}", ctx.id()))
        .kind(serenity::all::ChannelType::PublicThread);
    let thread = ctx.channel_id().create_thread(ctx.http(), builder).await?;
    thread.id.say(ctx.http(), "質問してみましょう").await?;

    let db_pool = &ctx.data().db_pool;
    let guild_id = ctx
        .guild_id()
        .context("想定外のエラーが発生しました")?
        .get();
    let channel_id = thread.id.get();
    let input = thread::InsertInput::new(guild_id, channel_id);
    Thread::insert(db_pool, &input).await?;

    Ok(())
}
