CREATE TABLE
  threads (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    guild_id BIGINT UNSIGNED NOT NULL,
    channel_id BIGINT UNSIGNED NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uk_threads_guild_id UNIQUE (guild_id),
    CONSTRAINT uk_threads_channel_id UNIQUE (channel_id)
  );
