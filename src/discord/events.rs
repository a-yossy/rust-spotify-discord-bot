pub mod messages;

use messages::Messages;
use poise::FrameworkContext;
use serenity::all::FullEvent;
use serenity::prelude::*;

use crate::types::discord::framework::{Data, Error};

pub struct Events;

impl Events {
    pub async fn handle(
        ctx: &Context,
        event: &FullEvent,
        framework: FrameworkContext<'_, Data, Error>,
        data: &Data,
    ) -> Result<(), Error> {
        match event {
            FullEvent::Message { new_message } => {
                Messages::handle(ctx, event, framework, data, &new_message).await?
            }
            _ => {}
        }

        Ok(())
    }
}
