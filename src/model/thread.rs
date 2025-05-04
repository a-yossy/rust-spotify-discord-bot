use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct Thread {
    pub id: u64,
    pub guild_id: u64,
    pub channel_id: u64,
    pub created_at: NaiveDateTime,
}
