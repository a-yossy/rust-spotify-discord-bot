use crate::types::discord::framework::Error;

#[poise::command(slash_command)]
pub async fn ping(ctx: super::Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;

    Ok(())
}
