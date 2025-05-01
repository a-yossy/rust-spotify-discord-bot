use crate::types::discord::framework::{Data, Error};

pub mod ping;

pub type Context<'a> = poise::Context<'a, Data, Error>;
