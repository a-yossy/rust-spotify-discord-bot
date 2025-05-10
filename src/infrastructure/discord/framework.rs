use poise::Framework as PoiseFramework;
use rig::{agent::Agent, providers::gemini::completion::CompletionModel};
use sqlx::MySqlPool;

use crate::{
    discord::{commands::Commands, events::Events},
    types::discord::framework::{Data, Error},
};

pub struct Framework;

impl Framework {
    pub fn get(
        llm_agent: Agent<CompletionModel>,
        db_pool: MySqlPool,
    ) -> PoiseFramework<Data, Error> {
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

        PoiseFramework::builder()
            .options(options)
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(Data::new(llm_agent, db_pool))
                })
            })
            .build()
    }
}
