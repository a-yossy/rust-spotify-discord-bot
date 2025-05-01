use poise::Framework;

use crate::{
    discord::commands::ping,
    types::discord::framework::{Data, Error},
};

pub fn get() -> Framework<Data, Error> {
    let options = poise::FrameworkOptions {
        commands: vec![ping::ping()],
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
