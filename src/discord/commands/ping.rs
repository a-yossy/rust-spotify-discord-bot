use crate::types::discord::framework::Error;

use super::Context;

pub async fn handle(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;

    Ok(())
}
