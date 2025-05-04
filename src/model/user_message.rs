use chrono::NaiveDateTime;

pub struct UserMessage {
    pub id: u64,
    pub thread_id: u64,
    pub message_id: u64,
    pub user_id: u64,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
