use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct AgentMessage {
    pub id: u64,
    pub user_message_id: u64,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
