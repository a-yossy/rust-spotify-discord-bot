use poise::Framework;

use crate::{
    discord::commands::Commands,
    types::discord::framework::{Data, Error},
};

pub fn get() -> Framework<Data, Error> {
    let options = poise::FrameworkOptions {
        commands: vec![Commands::ping(), Commands::random_music()],
        ..Default::default()
    };

    poise::Framework::builder()
        .options(options)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build()
}
