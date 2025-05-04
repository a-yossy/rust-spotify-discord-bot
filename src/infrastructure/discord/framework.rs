use poise::Framework;

use crate::{
    discord::{commands::Commands, events::Events},
    infrastructure::llm,
    types::discord::framework::{Data, Error},
};

pub fn get() -> Framework<Data, Error> {
    let options = poise::FrameworkOptions {
        commands: vec![
            Commands::ping(),
            Commands::random_music(),
            Commands::thread(),
        ],
        event_handler: |ctx, event, framework, data| {
            Box::pin(Events::handle(ctx, event, framework, data))
        },
        ..Default::default()
    };
    let llm_agent = llm::client::get();

    poise::Framework::builder()
        .options(options)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data::new(llm_agent))
            })
        })
        .build()
}
