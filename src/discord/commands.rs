use crate::types::discord::framework::{Data, Error};

pub mod ping;
pub mod random_music;
pub mod thread;

pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Commands;

impl Commands {
    #[poise::command(slash_command)]
    pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
        ping::handle(ctx).await
    }

    #[poise::command(slash_command)]
    pub async fn random_music(ctx: Context<'_>) -> Result<(), Error> {
        random_music::handle(ctx).await
    }

    #[poise::command(slash_command)]
    pub async fn thread(ctx: Context<'_>) -> Result<(), Error> {
        thread::handle(ctx).await
    }
}
