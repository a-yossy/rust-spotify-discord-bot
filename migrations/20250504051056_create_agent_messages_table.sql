CREATE TABLE
  agent_messages (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    message_id BIGINT UNSIGNED NOT NULL,
    user_message_id BIGINT UNSIGNED NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    CONSTRAINT fk_agent_messages_user_message_id FOREIGN KEY (user_message_id) REFERENCES user_messages (id),
    CONSTRAINT uk_agent_messages_message_id UNIQUE (message_id)
  );
